// import { IndexClient } from "candb-client-typescript/dist/IndexClient";
// import { ActorClient } from "candb-client-typescript/dist/ActorClient";

export function getIsLocal() {
  return process.env.REACT_APP_IS_LOCAL === "1";
}

// export function intializeCanDBIndexClient(): IndexClient<CanDBIndex> {
//   const host = isLocal ? "http://localhost:8000" : "https://ic0.app";
//   const canisterId = isLocal ? process.env.INDEX_CANISTER_ID : "<prod_canister_id>"; // TODO
//   return new IndexClient<CanDBIndex>({
//     IDL: CanDBIndexIDL,
//     canisterId, 
//     agentOptions: {
//       host,
//     },
//   });
// };

// TODO: Also partition client for a single canister.
// export function initializeCanDBPartitionClient(indexClient: IndexClient<CanDBIndex>)
//     : ActorClient<CanDBIndex, CanDBPartition>
// {
//   const host = isLocal ? "http://localhost:8000" : "https://ic0.app";
//   return new ActorClient<CanDBIndex, CanDBPartition>({
//     actorOptions: {
//       IDL: CanDBPartitionIDL,
//       agentOptions: {
//         host,
//       }
//     },
//     indexClient, 
//   });
// };