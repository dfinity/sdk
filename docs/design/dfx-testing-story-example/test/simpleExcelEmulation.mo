import R "mo:stdlib/result";
import P "mo:stdlib/prelude";
import Debug "mo:stdlib/debug";

import T "../src/types";
import A "../src/adapton";
import E "../src/eval";

actor simpleExcelEmulation {

  public func go() {

    let sheetExp : T.Eval.Exp =
      #sheet(
        #text("S"),
        [
          [ #nat(1),  #nat(2) ],
          [ #strictBinOp(#add,
                         #cellOcc(0,0),
                         #cellOcc(0,1)),
            #strictBinOp(#mul,
                         #nat(2),
                         #cellOcc(1,0)) ]
        ]);

    // Adapton maintains our dependence graph
    let actx : T.Adapton.Context = A.init(true);

    // create the initial Sheet datatype from the DSL expression above
    let s : T.Sheet.Sheet = {
      switch (E.evalExp(actx, null, sheetExp)) {
      case (#ok(#sheet(s))) s;
      case _ { P.unreachable() };
      }};

    // Demand that the sheet's results are fully refreshed
    ignore E.Sheet.refresh(actx, s);
    ignore E.Sheet.refresh(actx, s);
    A.assertLogEventLast(
      actx,
      #get(#tagTup(#text("S"), [#nat(1), #nat(1), #text("out")]), #ok(#nat(6)), [])
    );

    // Update the sheet by overwriting (0,0) with a new formula:
    ignore E.Sheet.update(actx, s, 0, 0,
                          #strictBinOp(#add, #nat(666), #cellOcc(0,1)));

    // Demand that the sheet's results are fully refreshed
    ignore E.Sheet.refresh(actx, s);
    ignore E.Sheet.refresh(actx, s);
    assert (s.errors.len() == 0);
    A.assertLogEventLast(
      actx,
      #get(#tagTup(#text("S"), [#nat(1), #nat(1), #text("out")]), #ok(#nat(1_340)), [])
    );

    // Update the sheet, creating a cycle at (0,0):
    ignore E.Sheet.update(actx, s, 0, 0, #cellOcc(0,0));
    ignore E.Sheet.refresh(actx, s);
    assert (s.errors.len() != 0);

    // Update the sheet, removing the cycle:
    ignore E.Sheet.update(actx, s, 0, 0,
                          #strictBinOp(#add, #nat(666), #cellOcc(0,1)));

    // Demand that the sheet's results are fully refreshed
    ignore E.Sheet.refresh(actx, s);
    ignore E.Sheet.refresh(actx, s);
    assert (s.errors.len() == 0);
    A.assertLogEventLast(
      actx,
      #get(#tagTup(#text("S"), [#nat(1), #nat(1), #text("out")]), #ok(#nat(1_340)), [])
    );
  };
};

//simpleExcelEmulation.go()
