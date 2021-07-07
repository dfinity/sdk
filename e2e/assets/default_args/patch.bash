#!/dev/null

patch invalid.mo <<EOF
@@ -1,3 +1,3 @@
 actor {
-
+bla
 }
EOF

cat invalid.mo

find .

dfx config canisters/e2e_project/main invalid.mo
dfx config defaults/build/args " --compacting-gc "
