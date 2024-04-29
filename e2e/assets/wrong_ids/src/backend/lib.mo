import xNat "mo:xtendedNumbers/NatX";
import Entity "mo:candb/Entity";
import Text "mo:base/Text";
import Debug "mo:base/Debug";
import Buffer "mo:base/Buffer";
import Principal "mo:base/Principal";
import Int "mo:base/Int";
import Nat32 "mo:base/Nat32";
import Nat8 "mo:base/Nat8";
import Blob "mo:base/Blob";
import Char "mo:base/Char";
import Nat64 "mo:base/Nat64";
import Array "mo:base/Array";
import Iter "mo:base/Iter";
import Time "mo:base/Time";
import Bool "mo:base/Bool";
import Reorder "mo:nacdb-reorder/Reorder";

module {
  // let phoneNumberVerificationCanisterId = "gzqxf-kqaaa-aaaak-qakba-cai"; // https://docs.nfid.one/developer/credentials/mobile-phone-number-credential

  // We will use that "-XXX" < "XXX" for any hex number XXX.

  func _toLowerHexDigit(v: Nat): Char {
    Char.fromNat32(Nat32.fromNat(
      if (v < 10) {
        Nat32.toNat(Char.toNat32('0')) + v;
      } else {
        Nat32.toNat(Char.toNat32('a')) - 10 + v;
      }
    ));
  };

  func _fromLowerHexDigit(c: Char): Nat {
    Nat32.toNat(
      if (c <= '9') {
        Char.toNat32(c) - Char.toNat32('0');
      } else {
        Char.toNat32(c) - (Char.toNat32('a') - 10);
      }
    );
  };

  func encodeBlob(g: Blob): Text {
    var result = "";
    for (b in g.vals()) {
      let b2 = Nat8.toNat(b);
      result #= Text.fromChar(_toLowerHexDigit(b2 / 16)) # Text.fromChar(_toLowerHexDigit(b2 % 16));
    };
    result;
  };

  func decodeBlob(t: Text): Blob {
    let buf = Buffer.Buffer<Nat8>(t.size() / 2);
    let c = t.chars();
    label r loop {
      let ?upper = c.next() else {
        break r;
      };
      let ?lower = c.next() else {
        Debug.trap("decodeBlob: wrong hex number");
      };
      let b = Nat8.fromNat(_fromLowerHexDigit(upper) * 16 + _fromLowerHexDigit(lower));
      buf.add(b);
    };
    Blob.fromArray(Buffer.toArray(buf));
  };

  public func encodeNat(n: Nat): Text {
    var n64 = Nat64.fromNat(n);
    let buf = Buffer.Buffer<Nat8>(8);
    for (i in Iter.range(0, 7)) {
      buf.add(Nat8.fromNat(Nat64.toNat(n64 % 256)));
      n64 >>= 8;
    };
    let blob = Blob.fromArray(Array.reverse(Buffer.toArray(buf)));
    encodeBlob(blob);
  };

  public func decodeNat(t: Text): Nat {
    let blob = decodeBlob(t);
    var result: Nat64 = 0;
    for (b in blob.vals()) {
      result <<= 8;
      result += xNat.from8To64(b);
    };
    Nat64.toNat(result);
  };

  // For integers less than 2**64 have the same lexigraphical sort order as the argument.
  public func encodeInt(n: Int): Text {
    assert n < 2**64;
    if (n >= 0) {
      encodeNat(Int.abs(n));
    } else {
      "-" # encodeNat(2**64 - Int.abs(n));
    };
  };

  public func decodeInt(t: Text): Int {
    let iter = t.chars();
    if (iter.next() == ?'-') {
      -(2**64 - decodeNat(Text.fromIter(iter)));
    } else {
      decodeNat(t);
    }
  };

  let ITEM_TYPE_LINK = 0;
  let ITEM_TYPE_MESSAGE = 1;
  let ITEM_TYPE_POST = 2;
  let ITEM_TYPE_FOLDER = 3;

  public type ItemDetails = {
    #link : Text;
    #message : ();
    #post : (); // save post text separately
    #folder : ();
  };

  public type ItemDataWithoutOwner = {
    price: Float;
    locale: Text;
    title: Text;
    description: Text;
    details: ItemDetails;
  };

  // TODO: Add `license` field?
  // TODO: Images.
  // TODO: Item version.
  // FIXME: Checking whether it's a folder in `order.mo` does not work.
  /// The type of stored item in DB.
  ///
  /// Stored at `"i/" # ID` attribute `"i"`.
  public type Item = {
    #owned : ItemData;
    #communal : {
      isFolder: Bool;
      timeStream: Reorder.Order;
      votesStream: Reorder.Order;
    };
  };

  public func isFolder(item: Item): Bool {
    switch (item) {
      case (#owned data) { data.item.details == #folder };
      case (#communal data) { data.isFolder };
    };
  };

  /// Used when requesting item data.
  public type ItemTransfer = {
    data: ItemData;
    communal: Bool;
  };

  /// Used when creating new items.
  public type ItemTransferWithoutOwner = {
    data: ItemDataWithoutOwner;
    communal: Bool;
  };

  public type ItemData = {
    creator: Principal;
    item: ItemDataWithoutOwner;
    edited: Bool;
  };

  /// One of voted-for variants of a communal item.
  ///
  /// Stored at `"r/" # ID` attribute `"i"`.
  ///
  /// TODO: Use it.
  public type ItemVariant = {
    item: ItemDataWithoutOwner;
    // itemRef: (Principal, Entity.SK);
  };

  // TODO: Does it make sense to keep `Streams` in lib?
  public type StreamsLinks = Nat;
  public let STREAM_LINK_SUBITEMS: StreamsLinks = 0; // folder <-> sub-items
  public let STREAM_LINK_SUBFOLDERS: StreamsLinks = 1; // folder <-> sub-folders
  public let STREAM_LINK_COMMENTS: StreamsLinks = 2; // item <-> comments
  public let STREAM_LINK_MAX: StreamsLinks = STREAM_LINK_COMMENTS;

  public type Streams = [?Reorder.Order];

  // TODO: messy order of the below functions

  func serializeItemDataWithoutOwnerToBuffer(
    buf: Buffer.Buffer<Entity.AttributeValuePrimitive>,
    item: ItemDataWithoutOwner,
  ) {
    buf.add(#int (switch (item.details) {
      case (#link v) { ITEM_TYPE_LINK };
      case (#message) { ITEM_TYPE_MESSAGE };
      case (#post _) { ITEM_TYPE_POST };
      case (#folder) { ITEM_TYPE_FOLDER };
    }));
    buf.add(#float(item.price));
    buf.add(#text(item.locale));
    buf.add(#text(item.title));
    buf.add(#text(item.description));
    switch (item.details) {
      case (#link v) {
        buf.add(#text v);
      };
      case _ {};
    };
  };

  func deserializeItemDataWithoutOwnerFromBuffer(arr: [Entity.AttributeValuePrimitive], current: {var pos: Nat})
    : ItemDataWithoutOwner
  {
    var kind: Nat = 0;
    var price = 0.0;
    var locale = "";
    var title = "";
    var description = "";
    var link = "";

    let res = label r: Bool {
      switch (arr[current.pos]) {
        case (#int v) {
          kind := Int.abs(v);
          current.pos += 1;
        };
        case _ { break r false };
      };
      switch (arr[current.pos]) {
        case (#float v) {
          price := v;
          current.pos += 1;
        };
        case _ { break r false; };
      };
      switch (arr[current.pos]) {
        case (#text v) {
          locale := v;
          current.pos += 1;
        };
        case _ { break r false; };
      };
      switch (arr[current.pos]) {
        case (#text v) {
          title := v;
          current.pos += 1;
        };
        case _ { break r false; };
      };
      switch (arr[current.pos]) {
        case (#text v) {
          description := v;
          current.pos += 1;
        };
        case _ { break r false; }
      };
      if (kind == ITEM_TYPE_LINK) { // TODO: Place it directly after `kind`?
        switch (arr[current.pos]) {
          case (#text v) {
            link := v;
            current.pos += 1;
          };
          case _ { break r false; };
        };
      };

      true;
    };

    if (not res) {
      Debug.trap("wrong item format");
    };
    {
      price = price;
      locale = locale;
      title = title;
      description = description;
      details = switch (kind) {
        case (0) { #link link };
        case (1) { #message };
        case (2) { #post };
        case (3) { #folder };
        case _ { Debug.trap("wrong item format"); }
      };
    };
  };

  func serializeItemToBuffer(
    buf: Buffer.Buffer<Entity.AttributeValuePrimitive>,
    item: Item,
  ) {
    switch (item) {
      case (#owned ownedItem) {
        buf.add(#int 0);
        buf.add(#text(Principal.toText(ownedItem.creator)));
        buf.add(#bool(ownedItem.edited));
        serializeItemDataWithoutOwnerToBuffer(buf, ownedItem.item);
      };
      case (#communal {isFolder: Bool; timeStream: Reorder.Order; votesStream: Reorder.Order}) {
        buf.add(#int 1);
        buf.add(#int(if (isFolder) { 1 } else { 0 }));
        buf.add(#text(Principal.toText(Principal.fromActor(timeStream.order.0))));
        buf.add(#int(timeStream.order.1));
        buf.add(#text(Principal.toText(Principal.fromActor(timeStream.reverse.0))));
        buf.add(#int(timeStream.reverse.1));
        buf.add(#text(Principal.toText(Principal.fromActor(votesStream.order.0))));
        buf.add(#int(votesStream.order.1));
        buf.add(#text(Principal.toText(Principal.fromActor(votesStream.reverse.0))));
        buf.add(#int(votesStream.reverse.1));
      };
    };
  };

  func deserializeItemFromBuffer(arr: [Entity.AttributeValuePrimitive], current: {var pos: Nat})
    : Item
  {
    label r {
      if (arr[current.pos] != #int 0) { // version marker
        break r;
      };
      current.pos += 1;
      switch (arr[current.pos]) {
        case (#int v) {
          current.pos += 1;
          switch (v) {
            case (0) {
              var creator = "";
              var edited = false;
              switch (arr[current.pos]) {
                case (#text c) {
                  creator := c;
                  current.pos += 1;
                };
                case _ { break r };
              };
              switch (arr[current.pos]) {
                case (#bool f) {
                  edited := f;
                  current.pos += 1;
                };
                case _ { break r };
              };
              return #owned({
                creator = Principal.fromText(creator);
                item = deserializeItemDataWithoutOwnerFromBuffer(arr, {var pos = current.pos});
                edited;
              });
            };
            case (1) {
              var folder = false;
              switch (arr[current.pos]) {
                case (#int 0) { folder := false; };
                case (#int 1) { folder := true; };
                case _ { break r };
              };
              current.pos += 1;
              var order = ("", 0);
              var reverse = ("", 0);
              var order2 = ("", 0);
              var reverse2 = ("", 0);
              switch ((arr[current.pos], arr[current.pos+1], arr[current.pos+2], arr[current.pos+3])) {
                case ((#text op1, #int on1, #text op2, #int on2)) {
                  order := (op1, Int.abs(on1));
                  reverse := (op2, Int.abs(on2));
                  current.pos += 4;
                };
                case _ { break r };
              };
              switch ((arr[current.pos], arr[current.pos+1], arr[current.pos+2], arr[current.pos+3])) {
                case ((#text op1, #int on1, #text op2, #int on2)) {
                  order2 := (op1, Int.abs(on1));
                  reverse2 := (op2, Int.abs(on2));
                  current.pos += 4;
                };
                case _ { break r };
              };
              return #communal {
                isFolder = folder;
                timeStream = { order = (actor(order.0), order.1); reverse = (actor(reverse.0), reverse.1) };
                votesStream = { order = (actor(order2.0), order2.1); reverse = (actor(reverse2.0), reverse2.1) };
              };
            };
            case _ { break r };
          }
        };
        case _ { break r };
      };
    };
    Debug.trap("wrong item format");
  };

  // FIXME: Should be `Item` instead of `ItemData`.
  public func serializeItem(item: Item): Entity.AttributeValue {
    let buf = Buffer.Buffer<Entity.AttributeValuePrimitive>(8);
    buf.add(#int 0); // version
    serializeItemToBuffer(buf, item);
    #tuple(Buffer.toArray(buf));
  };

  public func deserializeItem(attr: Entity.AttributeValue): Item {
    var pos = 0;
    label r switch (attr) {
      case (#tuple arr) {
        return deserializeItemFromBuffer(arr, {var pos});
      };
      case _ {};
    };
    Debug.trap("wrong item format");
  };

  public func serializeItemVariant(item: ItemVariant): Entity.AttributeValue {
    let buf = Buffer.Buffer<Entity.AttributeValuePrimitive>(8);
    buf.add(#int 0); // version
    serializeItemDataWithoutOwnerToBuffer(buf, item.item);
    #tuple(Buffer.toArray(buf));
  };

  public func deserializeItemVariant(attr: Entity.AttributeValue): ItemVariant {
    var pos = 0;
    switch (attr) {
      case (#tuple arr) {
        switch (arr[pos]) {
          case (#int v) {
            assert v == 0;
            pos += 1;
          };
          case _ { Debug.trap("wrong item format"); };
        };
        return {
          item = deserializeItemDataWithoutOwnerFromBuffer(arr, {var pos});
        };
      };
      case _ {
        Debug.trap("wrong item format");
      };
    };
  };

  public func serializeStreams(streams: Streams): Entity.AttributeValue {
    let buf = Buffer.Buffer<Entity.AttributeValuePrimitive>(18);
    for(item in streams.vals()) {
      switch (item) {
        case (?r) {
          buf.add(#text(Principal.toText(Principal.fromActor(r.order.0))));
          buf.add(#int(r.order.1));
          buf.add(#text(Principal.toText(Principal.fromActor(r.reverse.0))));
          buf.add(#int(r.reverse.1));
        };
        case null {
          buf.add(#int(-1));
        }
      }
    };
    #tuple(Buffer.toArray(buf));
  };

  public func deserializeStreams(attr: Entity.AttributeValue): Streams {
    let s = Buffer.Buffer<?Reorder.Order>(36);
    let #tuple arr = attr else {
      Debug.trap("programming error");
    };
    var i = 0;
    label w while (i != Array.size(arr)) {
      if (arr[i] == #int(-1)) {
        s.add(null);
        i += 1;
        continue w;
      };
      switch (arr[i], arr[i+1], arr[i+2], arr[i+3]) {
        case (#text c0, #int i0, #text c1, #int i1) {
          i += 4;
          s.add(
            ?{ order = (actor(c0), Int.abs(i0)); reverse = (actor(c1), Int.abs(i1)) },
          );
        };
        case _ {
          Debug.trap("programming error");
        }
      };
    };

    Buffer.toArray(s);
  };

  // TODO: Use this.
  public type Karma = {
    earnedVotes: Nat;
    remainingBonusVotes: Nat;
    lastBonusUpdated: Time.Time;
  };

  public func serializeKarma(karma: Karma): Entity.AttributeValue {
    #tuple([
      #int(0), // version
      #int(karma.earnedVotes),
      #int(karma.remainingBonusVotes),
      #int(karma.lastBonusUpdated),
    ]);
  };

  public func deserializeKarma(attr: Entity.AttributeValue): Karma {
    let res = label r {
      switch (attr) {
        case (#tuple arr) {
          let a: [var Nat] = Array.tabulateVar<Nat>(3, func _ = 0);
          switch (arr[0]) {
            case (#int v) {
              assert v == 0;
            };
            case _ { Debug.trap("Wrong karma version"); };
          };
          for (i in Iter.range(0,2)) {
            switch (arr[i+1]) {
              case (#int elt) {
                a[i] := Int.abs(elt);
              };
              case _ { break r; };
            };
            return {
              earnedVotes = a[0];
              remainingBonusVotes = a[1];
              lastBonusUpdated = a[2];
            };
          };
        };
        case _ { break r; };
      };
    };
    Debug.trap("wrong votes format");
  };

  public func onlyItemOwner(caller: Principal, item: Item) {
      switch (item) {
        case (#owned data) {
          if (caller != data.creator) {
            Debug.trap("can't change item owner");
          };
        };
        case (#communal _) {
          Debug.trap("can't directly edit a communal item");
        };
      };
  };

  /// More user info: Voting ///

  // TODO: Also store, how much votings were done.
  public type VotingScore = {
    points: Float; // Gitcoin score
    lastChecked: Time.Time;
    ethereumAddress: Text; // TODO: Store in binary
  };

  public func serializeVoting(voting: VotingScore): Entity.AttributeValue {
    let buf = Buffer.Buffer<Entity.AttributeValuePrimitive>(4);
    buf.add(#int 0); // version
    buf.add(#bool true);
    buf.add(#float(voting.points));
    buf.add(#int(voting.lastChecked));
    buf.add(#text(voting.ethereumAddress));
    #tuple(Buffer.toArray(buf));
  };

  public func deserializeVoting(attr: Entity.AttributeValue): VotingScore {
    var isScore: Bool = false;
    var points: Float = 0.0;
    var lastChecked: Time.Time = 0;
    var ethereumAddress: Text = "";

    let res = label r: Bool switch (attr) {
      case (#tuple arr) {
        var pos: Nat = 0;
        switch (arr[pos]) {
          case (#int v) {
            assert v == 0;
          };
          case _ { break r false };
        };
        pos += 1;
        switch (arr[pos]) {
          case (#bool v) {
            isScore := v;
          };
          case _ { break r false };
        };
        pos += 1;
        if (isScore) {
          switch (arr[pos]) {
            case (#float v) {
              points := v;
            };
            case _ { break r false };
          };
          pos += 1;
          switch (arr[pos]) {
            case (#int v) {
              lastChecked := v;
            };
            case _ { break r false };
          };
          pos += 1;
          switch (arr[pos]) {
            case (#text v) {
              ethereumAddress := v;
            };
            case _ { break r false };
          };
          pos += 1;
        };
        true;
      };
      case _ { break r false };
    };
    if (not res) {
      Debug.trap("cannot deserialize Voting");
    };
    {points; lastChecked; ethereumAddress};
  };

  /// Users ///

  public type User = {
    locale: Text;
    nick: Text;
    title: Text;
    description: Text;
    // TODO: long description
    link : Text;
  };

  public func serializeUser(user: User): Entity.AttributeValue {
    var buf = Buffer.Buffer<Entity.AttributeValuePrimitive>(6);
    buf.add(#int 0); // version
    buf.add(#text (user.locale));
    buf.add(#text (user.nick));
    buf.add(#text (user.title));
    buf.add(#text (user.description));
    buf.add(#text (user.link));
    #tuple (Buffer.toArray(buf));
  };

  public func deserializeUser(attr: Entity.AttributeValue): User {
    var locale = "";
    var nick = "";
    var title = "";
    var description = "";
    var link = "";
    let res = label r: Bool switch (attr) {
      case (#tuple arr) {
        var pos = 0;
        while (pos < arr.size()) {
          switch (pos) {
            case (0) {
              switch (arr[pos]) {
                case (#int v) {
                  assert v == 0; // version
                };
                case _ { break r false };
              };
            };
            case (1) {
              switch (arr[pos]) {
                case (#text v) {
                  locale := v;
                };
                case _ { break r false };
              };
            };
            case (2) {
              switch (arr[pos]) {
                case (#text v) {
                  nick := v;
                };
                case _ { break r false };
              };
            };
            case (3) {
              switch (arr[pos]) {
                case (#text v) {
                  title := v;
                };
                case _ { break r false };
              };
            };
            case (4) {
              switch (arr[pos]) {
                case (#text v) {
                  description := v;
                };
                case _ { break r false };
              };
            };
            case (5) {
              switch (arr[pos]) {
                case (#text v) {
                  link := v;
                };
                case _ { break r false };
              };
            };
            case _ { break r false; };
          };
          pos += 1;
        };
        true;
      };
      case _ {
        false;
      };
    };
    if (not res) {
      Debug.trap("wrong user format");
    };
    {
      locale = locale;
      nick = nick;
      title = title;
      description = description;
      link = link;
    };    
  };
}