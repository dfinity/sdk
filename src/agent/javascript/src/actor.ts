import { Buffer } from 'buffer/';
import { Agent } from './agent';
import { CanisterId } from './canisterId';
import assetCanister from './canisters/asset';
import managementCanister from './canisters/management';
import {
  QueryResponseStatus,
  RequestStatusResponseReplied,
  RequestStatusResponseStatus,
} from './http_agent_types';
import * as IDL from './idl';
import { GlobalInternetComputer } from './index';
import { RequestId, toHex as requestIdToHex } from './request_id';
import { BinaryBlob } from './types';

declare const window: GlobalInternetComputer;
declare const global: GlobalInternetComputer;
declare const self: GlobalInternetComputer;

function getDefaultAgent(): Agent {
  const agent =
    typeof window === 'undefined'
      ? typeof global === 'undefined'
        ? typeof self === 'undefined'
          ? undefined
          : self.ic.agent
        : global.ic.agent
      : window.ic.agent;

  if (!agent) {
    throw new Error('No Agent could be found.');
  }

  return agent;
}

/**
 * Configuration that can be passed to customize the Actor behaviour.
 */
export interface ActorConfig {
  canisterId: string | CanisterId;
  agent?: Agent;
  maxAttempts?: number;
  throttleDurationInMSecs?: number;
}

// TODO: move this to proper typing when Candid support TypeScript.
/**
 * A subclass of an actor. Actor class itself is meant to be a based class.
 */
export type ActorSubclass<T = Record<string, (...args: unknown[]) => Promise<unknown>>> = Actor & T;

/**
 *
 */
export enum CanisterInstallMode {
  Install = 'install',
  Reinstall = 'reinstall',
  Upgrade = 'upgrade',
}

/**
 * Internal metadata for actors. It's an enhanced version of ActorConfig with
 * some fields marked as required (as they are defaulted) and canisterId as
 * a CanisterId type.
 */
interface ActorMetadata {
  canisterId: CanisterId;
  service: IDL.ServiceClass;
  agent?: Agent;
  maxAttempts: number;
  throttleDurationInMSecs: number;
}

const kMetadataSymbol = Symbol();

/**
 * An actor base class. An actor is an object containing only functions that will
 * return a promise. These functions are derived from the IDL definition.
 */
export class Actor {
  public static getManagementCanister(config: Omit<ActorConfig, 'canisterId'>): ActorSubclass {
    return Actor.createActor(managementCanister, {
      ...config,
      canisterId: CanisterId.fromHex(''),
    });
  }

  public static interfaceOf(actor: Actor): IDL.ServiceClass {
    return actor[kMetadataSymbol].service;
  }

  public static canisterIdOf(actor: Actor): CanisterId {
    return actor[kMetadataSymbol].canisterId;
  }

  public static async install(
    fields: {
      module: BinaryBlob;
      mode?: CanisterInstallMode;
      arg?: BinaryBlob;
    },
    config: ActorConfig,
  ): Promise<void> {
    const mode = fields.mode || CanisterInstallMode.Install;
    // Need to transform the arg into a number array.
    const arg = fields.arg ? [...fields.arg] : [];
    // Same for module.
    const wasmModule = [...fields.module];
    const canisterId =
      typeof config.canisterId === 'string'
        ? CanisterId.fromText(config.canisterId)
        : config.canisterId;

    await this.getManagementCanister(config).install_code({
      mode: { [mode]: null },
      arg,
      wasm_module: wasmModule,
      canister_id: canisterId,
      compute_allocation: [],
      memory_allocation: [],
    });
  }

  public static async createAndInstallCanister(
    interfaceFactory: IDL.InterfaceFactory,
    fields: {
      module: BinaryBlob;
      arg?: BinaryBlob;
    },
    config?: Omit<ActorConfig, 'canisterId'>,
  ): Promise<ActorSubclass> {
    const { canister_id: canisterId } = (await this.getManagementCanister(
      config || {},
    ).create_canister()) as any;
    await this.install(
      {
        ...fields,
      },
      { ...config, canisterId },
    );

    return this.createActor(interfaceFactory, { ...config, canisterId });
  }

  public static createAssetCanisterActor(config: ActorConfig): ActorSubclass {
    return this.createActor(assetCanister, config);
  }

  public static createActorClass(interfaceFactory: IDL.InterfaceFactory): ActorConstructor {
    const service = interfaceFactory({ IDL });

    class CanisterActor extends Actor {
      [x: string]: (...args: unknown[]) => Promise<unknown>;

      constructor(config: ActorConfig) {
        const configWithDefaults = { ...DEFAULT_ACTOR_CONFIG, ...config };
        super({
          canisterId:
            typeof configWithDefaults.canisterId === 'string'
              ? CanisterId.fromText(configWithDefaults.canisterId)
              : configWithDefaults.canisterId,
          service,
          agent: configWithDefaults.agent,
          maxAttempts: configWithDefaults.maxAttempts,
          throttleDurationInMSecs: configWithDefaults.throttleDurationInMSecs,
        });
      }
    }

    for (const [methodName, func] of service._fields) {
      CanisterActor.prototype[methodName] = _createActorMethod(methodName, func);
    }

    return CanisterActor;
  }

  public static createActor(
    interfaceFactory: IDL.InterfaceFactory,
    configuration: ActorConfig,
  ): ActorSubclass {
    return new (this.createActorClass(interfaceFactory))(configuration);
  }

  private [kMetadataSymbol]: ActorMetadata;

  constructor(metadata: ActorMetadata) {
    this[kMetadataSymbol] = metadata;
  }

  // __createCanister(options?: {
  //   maxAttempts?: number;
  //   throttleDurationInMSecs?: number;
  // }): Promise<CanisterId>;
  // __setCanisterId(cid: CanisterId): void;
  // __canisterId(): string | undefined;
  // __getAsset(path: string): Promise<Uint8Array>;
}

// IDL functions can have multiple return values, so decoding always
// produces an array. Ensure that functions with single or zero return
// values behave as expected.
function decodeReturnValue(types: IDL.Type[], msg: BinaryBlob) {
  const returnValues = IDL.decode(types, Buffer.from(msg));
  switch (returnValues.length) {
    case 0:
      return undefined;
    case 1:
      return returnValues[0];
    default:
      return returnValues;
  }
}

const REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS = 500;
const DEFAULT_ACTOR_CONFIG = {
  maxAttempts: 30,
  throttleDurationInMSecs: REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS,
};

export type ActorConstructor = new (config: ActorConfig) => ActorSubclass;
export type ActorFactory = (config: ActorConfig) => ActorSubclass;

function _createActorMethod(
  methodName: string,
  func: IDL.FuncClass,
): (...args: unknown[]) => Promise<unknown> {
  if (func.annotations.includes('query')) {
    return async function (this: Actor, ...args: unknown[]) {
      const agent = this[kMetadataSymbol].agent || getDefaultAgent();
      const cid = this[kMetadataSymbol].canisterId;
      const arg = IDL.encode(func.argTypes, args) as BinaryBlob;

      const result = await agent.query(cid, { methodName, arg });

      switch (result.status) {
        case QueryResponseStatus.Rejected:
          throw new Error(
            `Query failed:\n` +
              `  Status: ${result.status}\n` +
              `  Message: ${result.reject_message}\n`,
          );

        case QueryResponseStatus.Replied:
          return decodeReturnValue(func.retTypes, result.reply.arg);
      }
    };
  } else {
    return async function (this: Actor, ...args: unknown[]) {
      const agent = this[kMetadataSymbol].agent || getDefaultAgent();
      const cid = this[kMetadataSymbol].canisterId;

      const { maxAttempts, throttleDurationInMSecs } = this[kMetadataSymbol];
      const arg = IDL.encode(func.argTypes, args) as BinaryBlob;
      const { requestId, response } = await agent.call(cid, { methodName, arg });

      if (!response.ok) {
        throw new Error(
          [
            'Call failed:',
            `  Method: ${methodName}(${args})`,
            `  Canister ID: ${cid.toHex()}`,
            `  Request ID: ${requestIdToHex(requestId)}`,
            `  HTTP status code: ${response.status}`,
            `  HTTP status text: ${response.statusText}`,
          ].join('\n'),
        );
      }

      return _requestStatusAndLoop(
        agent,
        requestId,
        status => {
          if (status.reply.arg !== undefined) {
            return decodeReturnValue(func.retTypes, status.reply.arg);
          } else if (func.retTypes.length === 0) {
            return undefined;
          } else {
            throw new Error(`Call was returned undefined, but type [${func.retTypes.join(',')}].`);
          }
        },
        maxAttempts,
        maxAttempts,
        throttleDurationInMSecs,
      );
    };
  }
}

async function _requestStatusAndLoop<T>(
  agent: Agent,
  requestId: RequestId,
  decoder: (response: RequestStatusResponseReplied) => T,
  attempts: number,
  maxAttempts: number,
  throttle: number,
): Promise<T> {
  const status = await agent.requestStatus({ requestId });

  switch (status.status) {
    case RequestStatusResponseStatus.Replied: {
      return decoder(status);
    }

    case RequestStatusResponseStatus.Unknown:
    case RequestStatusResponseStatus.Received:
    case RequestStatusResponseStatus.Processing:
      if (--attempts === 0) {
        throw new Error(
          `Failed to retrieve a reply for request after ${maxAttempts} attempts:\n` +
            `  Request ID: ${requestIdToHex(requestId)}\n` +
            `  Request status: ${status.status}\n`,
        );
      }

      // Wait a little, then retry.
      return new Promise(resolve => setTimeout(resolve, throttle)).then(() =>
        _requestStatusAndLoop(agent, requestId, decoder, attempts, maxAttempts, throttle),
      );

    case RequestStatusResponseStatus.Rejected:
      throw new Error(
        `Call was rejected:\n` +
          `  Request ID: ${requestIdToHex(requestId)}\n` +
          `  Reject code: ${status.reject_code}\n` +
          `  Reject text: ${status.reject_message}\n`,
      );
  }
}

// Make an actor from an actor interface.
//
// Allows for one HTTP agent for the lifetime of the actor:
//
// ```
// const actor = makeActor(actorInterface)({ agent });
// const reply = await actor.greet();
// ```
//
// or using a different HTTP agent for the same actor if necessary:
//
// ```
// const actor = makeActor(actorInterface);
// const reply1 = await actor(agent1).greet();
// const reply2 = await actor(agent2).greet();
// ```
export function makeActorFactory(actorInterfaceFactory: IDL.InterfaceFactory): ActorFactory {
  return (config: ActorConfig) => {
    return Actor.createActor(actorInterfaceFactory, config);
  };
}
