import R "mo:stdlib/result";
import P "mo:stdlib/prelude";

import T "../src/types";
import A "../src/adapton";
import E "../src/eval";

/*

This file reproduces the intro example from the Adapton Rust docs,
found here:

    https://docs.rs/adapton/0/adapton/#demand-driven-change-propagation

The DCG update behavior asserted here (see uses of
`assertLogEventLast` below) matches the cleaning/dirtying behavior
described in the link above, and in other documents and papers.

In particular, the same example is used in a recorded adapton talk,
from the http://adapton.org website.  The link above has slides with
still pictures from the talk.

*/

actor SimpleAdaptonDivByZero {

  public func go() {
    let ctx : T.Adapton.Context = A.init(true);

    // "cell 1 holds 42":
    let cell1 : T.Adapton.NodeId = assertOkPut(A.put(ctx, #nat(1), #nat(42)));
    A.assertLogEventLast
    (ctx, #put(#nat(1), #nat(42), []));

    // "cell 2 holds 2":
    let cell2 : T.Adapton.NodeId = assertOkPut(A.put(ctx, #nat(2), #nat(2)));
    A.assertLogEventLast
    (ctx, #put(#nat(2), #nat(2), []));

    // "cell 3 holds a suspended closure for this expression:
    //
    //   get(cell1) / get(cell2)
    //
    // ...and it is still unevaluated".
    //
    let cell3 : T.Adapton.NodeId = assertOkPut(
      A.putThunk(ctx, #nat(3),
                 E.closure(
                   null,
                   #strictBinOp(#div,
                                #get(#refNode(cell1)),
                                #get(#refNode(cell2))
                   )))
    );

    // "cell 4 holds a suspended closure for this expression:
    //
    //   if (get(cell2) == 0) { 0 }
    //   else { get(cell3) }
    //
    // ...and it is still unevaluated".
    //
    let cell4 : T.Adapton.NodeId = assertOkPut(
      A.putThunk(ctx, #nat(4),
                 E.closure(
                   null,
                   #ifCond(#strictBinOp(#eq,
                                        #get(#refNode(cell2)),
                                        #nat(0)),
                           #nat(0),
                           #get(#refNode(cell3)))))
    );

    // demand division:
    let res1 = assertOkGet(A.get(ctx, cell4));
    A.assertLogEventLast(ctx,
      #get(#nat(4), #ok(#nat(21)),
           [
             #evalThunk(
               #nat(4), #ok(#nat(21)),
               [#get(#nat(2), #ok(#nat(2)), []),
                #get(#nat(3), #ok(#nat(21)),
                     [
                       #evalThunk(
                         #nat(3), #ok(#nat(21)),
                         [
                           #get(#nat(1), #ok(#nat(42)), []),
                           #get(#nat(2), #ok(#nat(2)), [])
                         ])
                     ])
               ])
           ])
    );

    // "cell 2 holds 0":
    ignore A.put(ctx, #nat(2), #nat(0));
    A.assertLogEventLast
    (ctx,
     #put(#nat(2), #nat(0),
          [
            #dirtyIncomingTo(
              #nat(2),
              [
                #dirtyEdgeFrom(
                  #nat(3),
                  [
                    #dirtyIncomingTo(
                      #nat(3),
                      [
                        #dirtyEdgeFrom(
                          #nat(4),
                          [
                            #dirtyIncomingTo(#nat(4), [])
                          ])
                      ])
                  ]),
                #dirtyEdgeFrom(
                  #nat(4),
                  [
                    #dirtyIncomingTo(
                      #nat(4), [])
                  ])
              ])
          ])
    );

    // re-demand division:
    let res2 = assertOkGet(A.get(ctx, cell4));
    A.assertLogEventLast
    (ctx,
     #get(#nat(4), #ok(#nat(0)),
          [
            #cleanThunk(
              #nat(4), false,
              [
                #cleanEdgeTo(
                  #nat(2), false, [])
              ]),
            #evalThunk(
              #nat(4), #ok(#nat(0)),
              [
                #get(#nat(2), #ok(#nat(0)), [])
              ])
          ]));

    // "cell 2 holds 2":
    ignore A.put(ctx, #nat(2), #nat(2));
    A.assertLogEventLast
    (ctx,
     #put(#nat(2), #nat(2),
           [
             #dirtyIncomingTo(
               #nat(2),
               [
                 #dirtyEdgeFrom(
                   #nat(4),
                   [
                     #dirtyIncomingTo(
                       #nat(4), [])
                   ])
               ])
           ]));

    // re-demand division:
    let res3 = assertOkGet(A.get(ctx, cell4));
    A.assertLogEventLast
      (ctx,
       #get(#nat(4), #ok(#nat(21)),
            [
              #cleanThunk(
                #nat(4), false,
                [
                  #cleanEdgeTo(#nat(2), false, [])
                ]),
              #evalThunk(
                #nat(4), #ok(#nat(21)),
                [
                  #get(#nat(2), #ok(#nat(2)), []),
                  #get(#nat(3), #ok(#nat(21)),
                       [
                         #cleanThunk(
                           #nat(3), true,
                           [
                             #cleanEdgeTo(#nat(1), true, []),
                             #cleanEdgeTo(#nat(2), true, [])
                           ])
                       ])
                ])
            ]));
  };


  func assertOkPut(r:R.Result<T.Adapton.NodeId, T.Adapton.PutError>) : T.Adapton.NodeId {
    switch r {
      case (#ok(id)) { id };
      case _ { P.unreachable() };
    }
  };

  func assertOkGet(r:R.Result<T.Adapton.Result, T.Adapton.GetError>) : T.Eval.Result {
    switch r {
      case (#ok(res)) { res };
      case _ { P.unreachable() };
    }
  };
};

//SimpleAdaptonDivByZero.go();
