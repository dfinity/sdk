#!/dev/null

# add `canisters/e2e_project/args`, so that subsequent `dfx config` works
patch dfx.json <<EOF
@@ -4,7 +4,8 @@
   "canisters": {
     "e2e_project": {
       "type": "motoko",
-      "main": "src/e2e_project/main.mo"
+      "main": "src/e2e_project/main.mo",
+      "args" : ""
     },
     "e2e_project_assets": {
       "type": "assets",
EOF

dfx config canisters/e2e_project/args -- "--compacting-gcY"
dfx config defaults/build/args -- "--compacting-gcX"
