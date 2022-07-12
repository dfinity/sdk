#!/dev/null

patch dfx.json <<EOF
@@ -4,7 +4,7 @@
   "canisters": {
     "e2e_project": {
       "type": "motoko",
-      "main": "src/e2e_project/main.mo"
+      "main": "src/e2e_project/main.mo",
     },
     "e2e_project_assets": {
       "type": "assets",
EOF

cat <<<"$(jq '.canisters.e2e_project.args="--compacting-gcY"' dfx.json)" >dfx.json
cat <<<"$(jq '.defaults.build.args="--compacting-gcX"' dfx.json)" >dfx.json
