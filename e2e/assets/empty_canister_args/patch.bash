#!/dev/null

patch dfx.json <<EOF
@@ -4,7 +4,8 @@
   "canisters": {
     "e2e_project": {
       "type": "motoko",
-      "main": "src/e2e_project_backend/main.mo"
+      "main": "src/e2e_project_backend/main.mo",
+      "args" : ""
     },
     "e2e_project_frontend": {
       "type": "assets",
EOF

cat <<<"$(jq '.defaults.build.args="--error-detail 5 --compacting-gcX"' dfx.json)" >dfx.json
