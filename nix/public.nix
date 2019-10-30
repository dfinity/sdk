{ runCommandNoCC }:

runCommandNoCC "public-folder" {} ''
    mkdir -p $out
    cp -R ${../public}/. $out
''
