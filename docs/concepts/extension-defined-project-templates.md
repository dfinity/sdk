# Extension-Defined Project Templates

## Overview

An extension can define one or more project templates for `dfx new` to use.

A project template is a set of files that `dfx new` copies or patches into a new project.

For examples of project template files, see the [project_templates] directory in the SDK repository.

# Specification

The `project_templates` field in an extension's `extension.json` defines the project templates
included in the extension.  It is an object field mapping `project template name -> project template properties`.
These are the properties of a project template:

| Field                        | Type                      | Description                                                       |
|------------------------------|---------------------------|-------------------------------------------------------------------|
| `display`                    | String                    | Display name of the project template                              |
| `category`                   | String                    | Category for inclusion in `--backend` and `--frontend` CLI options |
| `requirements`               | Array of String           | Required project templates                                        |
| `post_create`                | String or Array of String | Command(s) to run after adding the canister to the project        |
| `post_create_spinner_message` | String                    | Message to display while running the post_create command         |
| `post_create_failure_warning` | String                    | Warning to display if the post_create command fails              |

Within the files distributed with the extension, the project template files are
located in the `project_templates/{project template name}` directory.

## The `display` field

The `display` field is a string that describes the project template.
`dfx new` will use this value for interactive selection of project templates.

## The `category` field

The `category` field is an array of strings that categorize the project template.
`dfx new` uses this field to determine whether to include this project template
as an option for the `--backend` and `-frontend` flags, as well as in interactive setup.

Valid values for the field:
- `frontend`
- `backend`
- `extra`
- `frontend-test`
- `support`

## The `requirements` field

The `requirements` field lists any project templates that `dfx new` must apply before this project template.
For example, many of the frontend templates depend on the `dfx_js_base` template, which adds
package.json and tsconfig.json to the project.

## The `post_create` field

The `post_create` field specifies a command or commands to run after adding the project template files to the project.
For example, the rust project template runs `cargo update` after adding the files.

## The `post_create_spinner_message` field

If this field is set, `dfx new` will display a spinner with this message while running the `post_create` command.

## The `post_create_failure_warning` field

If this field is present and the `post_create` command fails, `dfx new` will display this warning but won't stop creating the project.

[project_templates]: https://github.com/dfinity/sdk/tree/master/src/dfx/assets/project_templates
