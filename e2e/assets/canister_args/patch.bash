#!/dev/null

patch dfx.json <<EOF
@@ -4,7 +4,7 @@
   "canisters": {
     "e2e_project_backend": {
       "type": "motoko",
-      "main": "src/e2e_project_backend/main.mo"
+      "main": "src/e2e_project_backend/main.mo",
     },
     "e2e_project_frontend": {
       "type": "assets",
EOF

cat <<<"$(jq '.canisters.e2e_project_backend.args="--compacting-gcY"' dfx.json)" >dfx.json
cat <<<"$(jq '.defaults.build.args="--compacting-gcX"' dfx.json)" >dfx.json
