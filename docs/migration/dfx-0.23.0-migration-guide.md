# dfx 0.23.0 Migration Guide

## New frontend security header mechanism

It is possible to define custom headers for frontend assets in `.ic-assets.json5` files.
Previously, the default projects provided by `dfx new` included a default set of security headers like [this](https://github.com/dfinity/sdk/blob/b8ff661785e6979b2a155c525f91737ad058482a/src/dfx/assets/project_templates/vue/src/__frontend_name__/public/.ic-assets.json5).
While this is a solid default for now, it is likely that these headers will receive updates in the future and many projects will forget to update their security headers.
Additionally, it is unlikely that many projects actually improve over the defaults with the previous setup.

`dfx` version `0.23.0` introduces a new field `"security_policy"` to `.ic-assets.json5`,
which can automatically provide all the default security headers and updates that will be shipped in future versions of `dfx`.
You can check the default security headers with `dfx info security-policy`.

### If your `.ic-assets.json5` does not use any security headers

If your `.ic-assets.json5` does not use any security headers and you want to keep it that way, you don't need to do anything.
However, `dfx` will print a warning when you deploy your frontend that you didn't define a security policy.
If you want to get rid of the warning, you can add this to your `.ic-assets.json5`:

```json5
{
    "match": "**/*",
    "security_policy": "disabled"
}
```

This will instruct dfx to not add any security headers and to not print the warning anymore.

If your `.ic-assets.json5` does not use any security headers and you would like to add the default set of headers, you can add this to your `.ic-assets.json5`:

```json5
{
    "match": "**/*",
    "security_policy": "standard",

    // Uncomment to disable the warning about using the
    // standard security policy, if you understand the risk
    // "disable_security_policy_warning": true,
}
```

### If your `.ic-assets.json5` uses the default security headers

`dfx info security-policy` will print the default security headers.

If your `.ic-assets.json5` uses the default security headers without modification, replace the headers section (looks like [this](https://github.com/dfinity/sdk/blob/b8ff661785e6979b2a155c525f91737ad058482a/src/dfx/assets/project_templates/vue/src/__frontend_name__/public/.ic-assets.json5#L5-L48)) with this:

```json5
{
    "match": "**/*",
    "security_policy": "standard",

    // Uncomment to disable the warning about using the
    // standard security policy, if you understand the risk
    // "disable_security_policy_warning": true,
}
```

### If your `.ic-assets.json5` uses custom security headers

`dfx info security-policy` will print the default security headers.

If your `.ic-assets.json5` uses the default security headers with custom improvements, remove the headers that are the same as the default security headers.
Leave the headers that have custom improvements, and set `"security_policy": "hardened"`, like this:

```json5
{
    "match": "**/*",
    "security_policy": "hardened",
    "headers": {
        // this is an improvement over the default CSP
        "Content-Security-Policy": "super-secure-csp"
    }
}
```