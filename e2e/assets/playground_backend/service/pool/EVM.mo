// This is a generated Motoko binding.
// Please use `import service "ic:canister_id"` instead to call canisters on the IC if possible.

module {
  public type AccessListEntry = { storageKeys : [Text]; address : Text };
  public type Block = {
    miner : Text;
    totalDifficulty : ?Nat;
    receiptsRoot : Text;
    stateRoot : Text;
    hash : Text;
    difficulty : ?Nat;
    size : Nat;
    uncles : [Text];
    baseFeePerGas : ?Nat;
    extraData : Text;
    transactionsRoot : ?Text;
    sha3Uncles : Text;
    nonce : Nat;
    number : Nat;
    timestamp : Nat;
    transactions : [Text];
    gasLimit : Nat;
    logsBloom : Text;
    parentHash : Text;
    gasUsed : Nat;
    mixHash : Text;
  };
  public type BlockTag = {
    #Earliest;
    #Safe;
    #Finalized;
    #Latest;
    #Number : Nat;
    #Pending;
  };
  public type CallArgs = {
    transaction : TransactionRequest;
    block : ?BlockTag;
  };
  public type CallResult = { #Ok : Text; #Err : RpcError };
  public type ChainId = Nat64;
  public type ConsensusStrategy = {
    #Equality;
    #Threshold : { min : Nat8; total : ?Nat8 };
  };
  public type EthMainnetService = {
    #Alchemy;
    #Llama;
    #BlockPi;
    #Cloudflare;
    #PublicNode;
    #Ankr;
  };
  public type EthSepoliaService = {
    #Alchemy;
    #BlockPi;
    #PublicNode;
    #Ankr;
    #Sepolia;
  };
  public type FeeHistory = {
    reward : [[Nat]];
    gasUsedRatio : [Float];
    oldestBlock : Nat;
    baseFeePerGas : [Nat];
  };
  public type FeeHistoryArgs = {
    blockCount : Nat;
    newestBlock : BlockTag;
    rewardPercentiles : ?Blob;
  };
  public type FeeHistoryResult = { #Ok : FeeHistory; #Err : RpcError };
  public type GetBlockByNumberResult = { #Ok : Block; #Err : RpcError };
  public type GetLogsArgs = {
    fromBlock : ?BlockTag;
    toBlock : ?BlockTag;
    addresses : [Text];
    topics : ?[Topic];
  };
  public type GetLogsResult = { #Ok : [LogEntry]; #Err : RpcError };
  public type GetTransactionCountArgs = { address : Text; block : BlockTag };
  public type GetTransactionCountResult = { #Ok : Nat; #Err : RpcError };
  public type GetTransactionReceiptResult = {
    #Ok : ?TransactionReceipt;
    #Err : RpcError;
  };
  public type HttpHeader = { value : Text; name : Text };
  public type HttpOutcallError = {
    #IcError : { code : RejectionCode; message : Text };
    #InvalidHttpJsonRpcResponse : {
      status : Nat16;
      body : Text;
      parsingError : ?Text;
    };
  };
  public type InstallArgs = {
    logFilter : ?LogFilter;
    demo : ?Bool;
    manageApiKeys : ?[Principal];
  };
  public type JsonRpcError = { code : Int64; message : Text };
  public type L2MainnetService = {
    #Alchemy;
    #Llama;
    #BlockPi;
    #PublicNode;
    #Ankr;
  };
  public type LogEntry = {
    transactionHash : ?Text;
    blockNumber : ?Nat;
    data : Text;
    blockHash : ?Text;
    transactionIndex : ?Nat;
    topics : [Text];
    address : Text;
    logIndex : ?Nat;
    removed : Bool;
  };
  public type LogFilter = {
    #ShowAll;
    #HideAll;
    #ShowPattern : Regex;
    #HidePattern : Regex;
  };
  public type Metrics = {
    cyclesWithdrawn : Nat;
    responses : [((Text, Text, Text), Nat64)];
    errNoPermission : Nat64;
    inconsistentResponses : [((Text, Text), Nat64)];
    cyclesCharged : [((Text, Text), Nat)];
    requests : [((Text, Text), Nat64)];
    errHttpOutcall : [((Text, Text), Nat64)];
    errHostNotAllowed : [(Text, Nat64)];
  };
  public type MultiCallResult = {
    #Consistent : CallResult;
    #Inconsistent : [(RpcService, CallResult)];
  };
  public type MultiFeeHistoryResult = {
    #Consistent : FeeHistoryResult;
    #Inconsistent : [(RpcService, FeeHistoryResult)];
  };
  public type MultiGetBlockByNumberResult = {
    #Consistent : GetBlockByNumberResult;
    #Inconsistent : [(RpcService, GetBlockByNumberResult)];
  };
  public type MultiGetLogsResult = {
    #Consistent : GetLogsResult;
    #Inconsistent : [(RpcService, GetLogsResult)];
  };
  public type MultiGetTransactionCountResult = {
    #Consistent : GetTransactionCountResult;
    #Inconsistent : [(RpcService, GetTransactionCountResult)];
  };
  public type MultiGetTransactionReceiptResult = {
    #Consistent : GetTransactionReceiptResult;
    #Inconsistent : [(RpcService, GetTransactionReceiptResult)];
  };
  public type MultiSendRawTransactionResult = {
    #Consistent : SendRawTransactionResult;
    #Inconsistent : [(RpcService, SendRawTransactionResult)];
  };
  public type Provider = {
    access : RpcAccess;
    alias : ?RpcService;
    chainId : ChainId;
    providerId : ProviderId;
  };
  public type ProviderError = {
    #TooFewCycles : { expected : Nat; received : Nat };
    #InvalidRpcConfig : Text;
    #MissingRequiredProvider;
    #ProviderNotFound;
    #NoPermission;
  };
  public type ProviderId = Nat64;
  public type Regex = Text;
  public type RejectionCode = {
    #NoError;
    #CanisterError;
    #SysTransient;
    #DestinationInvalid;
    #Unknown;
    #SysFatal;
    #CanisterReject;
  };
  public type RequestCostResult = { #Ok : Nat; #Err : RpcError };
  public type RequestResult = { #Ok : Text; #Err : RpcError };
  public type RpcAccess = {
    #Authenticated : { publicUrl : ?Text; auth : RpcAuth };
    #Unauthenticated : { publicUrl : Text };
  };
  public type RpcApi = { url : Text; headers : ?[HttpHeader] };
  public type RpcAuth = {
    #BearerToken : { url : Text };
    #UrlParameter : { urlPattern : Text };
  };
  public type RpcConfig = {
    responseConsensus : ?ConsensusStrategy;
    responseSizeEstimate : ?Nat64;
  };
  public type RpcError = {
    #JsonRpcError : JsonRpcError;
    #ProviderError : ProviderError;
    #ValidationError : ValidationError;
    #HttpOutcallError : HttpOutcallError;
  };
  public type RpcService = {
    #EthSepolia : EthSepoliaService;
    #BaseMainnet : L2MainnetService;
    #Custom : RpcApi;
    #OptimismMainnet : L2MainnetService;
    #ArbitrumOne : L2MainnetService;
    #EthMainnet : EthMainnetService;
    #Provider : ProviderId;
  };
  public type RpcServices = {
    #EthSepolia : ?[EthSepoliaService];
    #BaseMainnet : ?[L2MainnetService];
    #Custom : { chainId : ChainId; services : [RpcApi] };
    #OptimismMainnet : ?[L2MainnetService];
    #ArbitrumOne : ?[L2MainnetService];
    #EthMainnet : ?[EthMainnetService];
  };
  public type SendRawTransactionResult = {
    #Ok : SendRawTransactionStatus;
    #Err : RpcError;
  };
  public type SendRawTransactionStatus = {
    #Ok : ?Text;
    #NonceTooLow;
    #NonceTooHigh;
    #InsufficientFunds;
  };
  public type Topic = [Text];
  public type TransactionReceipt = {
    to : ?Text;
    status : ?Nat;
    transactionHash : Text;
    blockNumber : Nat;
    from : Text;
    logs : [LogEntry];
    blockHash : Text;
    type_ : Text;
    transactionIndex : Nat;
    effectiveGasPrice : Nat;
    logsBloom : Text;
    contractAddress : ?Text;
    gasUsed : Nat;
  };
  public type TransactionRequest = {
    to : ?Text;
    gas : ?Nat;
    maxFeePerGas : ?Nat;
    gasPrice : ?Nat;
    value : ?Nat;
    maxFeePerBlobGas : ?Nat;
    from : ?Text;
    type_ : ?Text;
    accessList : ?[AccessListEntry];
    nonce : ?Nat;
    maxPriorityFeePerGas : ?Nat;
    blobs : ?[Text];
    input : ?Text;
    chainId : ?Nat;
    blobVersionedHashes : ?[Text];
  };
  public type ValidationError = { #Custom : Text; #InvalidHex : Text };
  public type Self = actor {
    eth_call : shared (
        RpcServices,
        ?RpcConfig,
        CallArgs,
      ) -> async MultiCallResult;
    eth_feeHistory : shared (
        RpcServices,
        ?RpcConfig,
        FeeHistoryArgs,
      ) -> async MultiFeeHistoryResult;
    eth_getBlockByNumber : shared (
        RpcServices,
        ?RpcConfig,
        BlockTag,
      ) -> async MultiGetBlockByNumberResult;
    eth_getLogs : shared (
        RpcServices,
        ?RpcConfig,
        GetLogsArgs,
      ) -> async MultiGetLogsResult;
    eth_getTransactionCount : shared (
        RpcServices,
        ?RpcConfig,
        GetTransactionCountArgs,
      ) -> async MultiGetTransactionCountResult;
    eth_getTransactionReceipt : shared (
        RpcServices,
        ?RpcConfig,
        Text,
      ) -> async MultiGetTransactionReceiptResult;
    eth_sendRawTransaction : shared (
        RpcServices,
        ?RpcConfig,
        Text,
      ) -> async MultiSendRawTransactionResult;
    getMetrics : shared query () -> async Metrics;
    getNodesInSubnet : shared query () -> async Nat32;
    getProviders : shared query () -> async [Provider];
    getServiceProviderMap : shared query () -> async [(RpcService, ProviderId)];
    request : shared (RpcService, Text, Nat64) -> async RequestResult;
    requestCost : shared query (
        RpcService,
        Text,
        Nat64,
      ) -> async RequestCostResult;
    updateApiKeys : shared [(ProviderId, ?Text)] -> async ();
  }
}

