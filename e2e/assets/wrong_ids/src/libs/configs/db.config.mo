import Nac "mo:nacdb/NacDB";

module {
    public let dbOptions/*: Nac.DBOptions*/ = {
        moveCap = #usedMemory 500_000_000;
        hardCap = ?10_000;
        partitionCycles = 228_000_000_000;
        timeout = 20_000_000_000; // 20 sec
        createDBQueueLength = 60;
        insertQueueLength = 60;
    };
}