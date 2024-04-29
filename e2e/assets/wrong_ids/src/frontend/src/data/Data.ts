import { Principal } from "@dfinity/principal";
import {idlFactory as canDBPartitionIdl } from "../../../declarations/CanDBPartition";
import { CanDBPartition, ItemData, ItemTransfer, Streams } from "../../../declarations/CanDBPartition/CanDBPartition.did";
import { idlFactory as nacDBPartitionIdl } from "../../../declarations/NacDBPartition";
import { NacDBPartition } from "../../../declarations/NacDBPartition/NacDBPartition.did"; // FIXME
import { Actor, Agent, HttpAgent } from "@dfinity/agent";
import { idlFactory as canDBIndexIdl } from "../../../declarations/CanDBIndex";
import { CanDBIndex } from "../../../declarations/CanDBIndex/CanDBIndex.did";
import { useContext } from "react";
import { AuthContext } from '../component/auth/use-auth-client';

const STREAM_LINK_SUBITEMS = 0; // folder <-> sub-items
const STREAM_LINK_SUBFOLDERS = 1; // folder <-> sub-folders
const STREAM_LINK_COMMENTS = 2; // item <-> comments
// const STREAM_LINK_MAX = STREAM_LINK_COMMENTS;

export type ItemRef = {
    canister: Principal;
    id: number;
};

// TODO: This and the following functions probably should be not here.
export function parseItemRef(itemId: string): ItemRef {
    const a = itemId.split('@', 2);
    return {canister: Principal.fromText(a[1]), id: parseInt(a[0])};
}

export function serializeItemRef(item: ItemRef): string {
    return item.id + "@" + item.canister;
}

function _unwrap<T>(v: T[]): T | undefined {
    // TODO: simplify for greater performance
    return v === undefined || v.length === 0 ? undefined : v[0];
}

export class ItemDB {
    agent?: Agent; // should be `defaultAgent`
    itemRef: ItemRef;
    item: ItemTransfer;
    streams: Streams | undefined;
    streamsRev: Streams | undefined;
    communal: boolean;
    protected constructor(agent: Agent, itemId: string) {
        this.agent = agent;
        this.itemRef = parseItemRef(itemId);
    }
    /// `"t" | "v"` - time, votes,.
    static async create(agent: Agent, itemId: string, kind: "t" | "v"): Promise<ItemDB> {
        const obj = new ItemDB(agent, itemId);
        const client = Actor.createActor(canDBPartitionIdl, {canisterId: obj.itemRef.canister, agent});
        // TODO: Retrieve both by one call?
        const [item, streams, streamsRev] = await Promise.all([
            client.getItem(BigInt(obj.itemRef.id)),
            client.getStreams(BigInt(obj.itemRef.id), "s" + kind),
            client.getStreams(BigInt(obj.itemRef.id), "rs" + kind),
        ]) as [ItemTransfer[] | [], Streams[] | [], Streams[] | []];
        obj.item = item[0]; // TODO: if no such item
        obj.streams = _unwrap(streams);
        obj.streamsRev = _unwrap(streamsRev);
        return obj;
    }
    async locale(): Promise<string> {
        return this.item.data.item.locale;
    }
    async title(): Promise<string> {
        return this.item.data.item.title;
    }
    async description(): Promise<string> {
        return this.item.data.item.description;
    }
    async details() {
        return this.item.data.item.details;
    }
    async creator(): Promise<Principal> {
        return this.item.data.creator;
    }
    async postText(): Promise<string | undefined> {
        const client = Actor.createActor(canDBPartitionIdl, {canisterId: this.itemRef.canister, agent: this.agent});
        const t = (await client.getAttribute({sk: "i/" + this.itemRef.id}, "t") as any)[0]; // TODO: error handling
        return t === undefined ? undefined : Object.values(t)[0] as string;
    }
    // TODO: duplicate code with AllItems
    private async aList(outerCanister, outerKey, opts?: {lowerBound?: string, limit?: number})
        : Promise<{order: string, id: ItemRef, item: ItemTransfer}[]>
    {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        const client: NacDBPartition = Actor.createActor(nacDBPartitionIdl, {canisterId: outerCanister, agent: this.agent });
        const {canister: innerPart, key: innerKey} = (await client.getInner({outerKey}) as any)[0]; // TODO: error handling
        const client2 = Actor.createActor(nacDBPartitionIdl, {canisterId: innerPart, agent: this.agent });
        const items = ((await client2.scanLimitInner({innerKey, lowerBound, upperBound: "x", dir: {fwd: null}, limit: BigInt(limit)})) as any).results as
            [[string, {text: string}]] | [];
        const items1aa = items.length === 0 ? [] : items.map(x => ({key: x[0], text: x[1].text}));
        const items1a: {order: string, principal: string, id: number}[] = items1aa.map(x => {
            const m = x.text.match(/^([0-9]*)@(.*)$/);
            return {order: x.key, principal: m[2], id: Number(m[1])};
        });
        const items2 = items1a.map(({order, principal, id}) => { return {canister: Principal.from(principal), id, order} });
        const items3 = items2.map(id => (async () => {
            const part: CanDBPartition = Actor.createActor(canDBPartitionIdl, {canisterId: id.canister, agent: this.agent });
            return {order: id.order, id, item: await part.getItem(BigInt(id.id))};
        })());
        const items4 = await Promise.all(items3);
        return items4.map(({order, id, item}) => ({
            order,
            id,
            item: item[0],
        }));
    }
    async subFolders(opts?: {lowerBound?: string, limit?: number}): Promise<{order: string, id: ItemRef, item: ItemTransfer}[]> {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        if (this.agent === undefined) {
            return undefined;
        }
        if (this.streams === undefined || _unwrap(this.streams[STREAM_LINK_SUBFOLDERS]) === undefined) {
            return [];
        }
        const [outerCanister, outerKey] = _unwrap(this.streams[STREAM_LINK_SUBFOLDERS]).order;
        return await this.aList(outerCanister, outerKey, {lowerBound, limit})
    }
    async superFolders(opts?: {lowerBound?: string, limit?: number}): Promise<{order: string, id: ItemRef, item: ItemTransfer}[]> {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        if (this.agent === undefined) {
            return undefined;
        }
        if (this.streamsRev === undefined) {
            return [];
        }
        const stream = (this.item.data.item.details as any).folder !== undefined
            ? _unwrap(this.streamsRev[STREAM_LINK_SUBFOLDERS]) : _unwrap(this.streamsRev[STREAM_LINK_SUBITEMS]);
        if (stream === undefined) {
            return [];
        }
        const [outerCanister, outerKey] = stream.order;
        return await this.aList(outerCanister, outerKey, {lowerBound, limit})
    }
    async items(opts?: {lowerBound?: string, limit?: number}): Promise<{order: string, id: ItemRef, item: ItemTransfer}[]> {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        if (this.agent === undefined) {
            return undefined;
        }
        if (this.streams === undefined || _unwrap(this.streams[STREAM_LINK_SUBITEMS]) === undefined) {
            return [];
        }
        const [outerCanister, outerKey] = _unwrap(this.streams[STREAM_LINK_SUBITEMS]).order;
        return await this.aList(outerCanister, outerKey, {lowerBound, limit})
    }
    async comments(opts?: {lowerBound?: string, limit?: number}): Promise<{order: string, id: ItemRef, item: ItemTransfer}[]> {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        if (this.agent === undefined) {
            return undefined;
        }
        if (this.streams === undefined || _unwrap(this.streams[STREAM_LINK_COMMENTS]) === undefined) {
            return [];
        }
        const [outerCanister, outerKey] = _unwrap(this.streams[STREAM_LINK_COMMENTS]).order
        return await this.aList(outerCanister, outerKey, {lowerBound, limit})
    }
    async antiComments(opts?: {lowerBound?: string, limit?: number}): Promise<{order: string, id: ItemRef, item: ItemTransfer}[]> {
        const {lowerBound, limit} = opts !== undefined ? opts : {lowerBound: "", limit: 5};
        if (this.agent === undefined) {
            return undefined;
        }
        if (this.streamsRev === undefined || _unwrap(this.streamsRev[STREAM_LINK_COMMENTS]) === undefined) {
            return [];
        }
        const [outerCanister, outerKey] = _unwrap(this.streamsRev[STREAM_LINK_COMMENTS]).order
        return await this.aList(outerCanister, outerKey, {lowerBound, limit})
    }
}

export async function loadTotalVotes(agent: Agent, parent: ItemRef, child: ItemRef): Promise<{up: number, down: number}> {
    let pk = `user`;
    const canDBIndex: CanDBIndex = Actor.createActor(canDBIndexIdl, {canisterId: process.env.CANISTER_ID_CANDBINDEX!, agent})
    let results = await canDBIndex.getFirstAttribute(
        pk,
        {sk: `w/${parent.id}/${child.id}`, key: "v"},
    );
    if (results.length === 0) {
        return {up: 0, down: 0};
    }
    const tuple = (results[0][1][0] as any).tuple;
    return { up: tuple[0].int, down: tuple[1].int };
}

export async function loadUserVote(agent: Agent, principal: Principal, parent: ItemRef, child: ItemRef): Promise<number> {
    let pk = `user`;
    const canDBIndex: CanDBIndex = Actor.createActor(canDBIndexIdl, {canisterId: process.env.CANISTER_ID_CANDBINDEX!, agent})
    let results = await canDBIndex.getFirstAttribute(
        pk,
        {sk: `v/${principal.toString()}/${parent.id}/${child.id}`, key: "v"},
    );
    return results.length === 0 ? 0 : (results[0][1][0] as any).int;
}