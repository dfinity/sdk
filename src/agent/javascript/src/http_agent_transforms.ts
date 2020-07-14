import { Endpoint, HttpAgentRequest, HttpAgentRequestTransformFn } from './http_agent_types';
import { makeNonce, Nonce } from './types';

export function makeNonceTransform(nonceFn: () => Nonce = makeNonce): HttpAgentRequestTransformFn {
  return async (request: HttpAgentRequest) => {
    if (request.endpoint !== Endpoint.Read) {
      request.body.nonce = nonceFn();
    }
  };
}
