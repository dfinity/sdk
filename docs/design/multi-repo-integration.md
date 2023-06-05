# Overview

This design document explore how to setup and integrate the different
repositories that make the SDK.

# Background

There are currently multiple repos that the SDK team manages every day;

1.  The SDK itself (dfx);

2.  Documentation

3.  Examples

4.  Rust CDK

This does not include the repositories that are covered by sub-teams or
with collaboration with the Language team, like Candid, Cancan and
LinkedUp.

Also, Rust and JavaScript agents are going to move to their own
repositories, as well as the bootstrap code.

The SDK repo will become the DFX repo, and run the integration with
other repos.

# Expected User/Developer Experience

Developers on the SDK should be able to focus on the repo they work in,
without having multiple technologies and tools that take a lot of
cognitive load or time to work.

# Detailed Design

## Dependency Graph

The most important aspect of splitting the SDK into multiple repos is to
have a clear dependency graph between repositories. Given how we want to
separate the main SDK repo in the short term future, this graph could
look like this:

<figure>
<img src="dependency_graph" alt="dependency_graph" />
</figure>

We can emerge from this clear leaf nodes. These leafs don’t benefit from
using a complex CI/CD and build system; they’re mono-languistic, focused
and simple libraries used by more complex products. And distributed
using their own package managers (npm for JavaScript, crates.io for
Rust).

## Conventional Commits

All repos should follow (and enforce) conventional commits.

## Branching

Three branches should exist in every new repository;

1.  `next` which is the next major version, can have breaking changes.

2.  `minor` which is the next minor version, cannot have breaking
    changes but can add new features. This branch is optional

3.  `x.y` which is the stable release (e.g. `0.5`) and may only contain
    patches deemed important to the current or previous version (e.g.
    security fixes).

## Versioning

Repos should fit whatever versioning scheme seems to fit.

The only constraints would be with Agents, as they should version with
the spec they follow in mind, as they’ll be more or less changed with
spec changes. So an Agent following spec 0.8 should have version 0.8.0,
then increment patch numbers with each releases.

## Tests

Following a few principles:

1.  Each repo should test their own code using unit tests.

2.  Each repo should not allow PRs to be merged if unit tests are
    failing.

3.  Repos should NOT test their dependencies' code using unit tests
    (this is currently not the case).

4.  Each repos should have a set of integration tests with their
    immediate upstream dependency.

5.  True end-to-end tests become clearly the responsibility of source
    nodes; Docs (testing of their tutorials), Examples and SDK (current
    e2e suite).

6.  Ideally, each repos should have a set of tests for preventing (or
    deliberately allowing) breaking changes. Such tests could include
    integration testing with the downstream repository.

### API Regression Testing

#### Rust

There is currently a proposal to have RustDoc outputs JSON (see
[here](https://github.com/rust-lang/rfcs/pull/2963)) as a backend. This
proposal would allow us to setup an API extractor that works as a
backward-compatiblity test, similar in spirit to
[semverver](https://github.com/rust-dev-tools/rust-semverver) but more
standard and better supported (semverver hasn’t been working
consistently for months).

#### TypeScript

Microsoft has been publishing API-Extractor for a while. This generates
a JSON file that can be used to validate any API changes.

#### Other

Other languages should have a way to export or test their API, depending
on the language itself. For example, a list of expected APIs in a linked
object if the language does not have good support for API extraction
(e.g. C++).

# Documentation

CONTRIBUTING docs should be maintained in sync between repos. The master
repo for these templates should be either Docs, Common or a new repo for
organization specific documentation.

## Releases

Each package would be released on their own package manager on a
different (but hopefully in sync) schedule as the other packages. For
example, JavaScript code should be released on NPM, while Rust code on
crates.io.

Each release should be tagged on GitHub and could be automated easily
compared to DFX itself. Since each repo should follow conventional
commits, release notes could be automated for each repo, with the major
SDK repo being the grab all overview of all documented releases.

# Work Breakdown

The first step would be separate the different repos and validate

The current best repos to do this would be (in order):

1.  Rust Agent. This will validate that we can still use Hydra and Nix
    with a crate dependency that depends on a github repo.

2.  JavaScript Agent into 1 repo 2 packages; types and agent. This will
    straighten up the dependencies between DFX, the Agent and the
    packages we publish.

3.  Bootstrap. This will remove the direct link from DFX → JavaScript
    Agent. This will also be a good point to add browser tests to the
    Bootstrap repo.

At this point this design will be validated as viable. New repos can be
added, but the current repos should remain mostly the same.
