{ actorscript, dfinity }:

let subdir = "assets/v0_2_0_files"; in

{
  inherit subdir;

  copy = ''
    mkdir -p ${subdir}
    cp ${dfinity.rust-workspace}/bin/{client,nodemanager} ${subdir}
    cp ${actorscript.asc}/bin/asc ${subdir}
    cp ${actorscript.as-ide}/bin/as-ide ${subdir}
    cp ${actorscript.didc}/bin/didc ${subdir}
    cp ${actorscript.rts}/rts/as-rts.wasm ${subdir}
  '';
}
