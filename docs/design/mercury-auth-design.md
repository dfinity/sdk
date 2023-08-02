# Overview

An agent interacts with the IC by sending requests. These requests must
be either anonymous or authenticated for a given user identity (or
principal). Request authentication uses digital signatures: the agent
holds a private (signing) key on behalf of the user, which it uses to
sign the *request id* of each request sent to the IC. The agent then
sends the request to the IC along with the user’s public (verification)
key, which is linked to the user identity. The IC verifies the signature
with respect to the public key, and (if the signature verifies and the
public key is linked to the principal claimed by the agent) provides the
request to the canister, setting the *caller* of the request to the
claimed identity.

For request authentication to be secure and usable, the agent must
enable the user to handle the keys that are linked to the identity in a
way that is both secure (i.e. access to private keys is protected) as
well as usable (i.e. allows the user to backup the key, use it on
multiple devices, etc.) While the pre-Mercury implementation of the
agent does support request authentication, it falls short in terms of
identity management.

# Background

## Problem Statement

The proposed design ensures that user keys are stored securely, while at
the same time allowing the user to manage access to its identity safely
as well as on mulitple devices.

## Requirements

The key management in the browser agent must support the following:

-   **Secure key storage**: User keys must be stored in a way that
    protects them from being extracted from the canister front end.

-   **Safe and usable identity management**:

    -   Users can recover access to their identity even in case of
        device failure or loss.

    -   Users can use their identity from multiple devices.

    -   Users can use their identity on multiple canisters on the same
        device, once that device is authorized.

# Expected User/Developer Experience

When a user, for the very first time, interacts with a canister hosted
on the IC, the front page shows an option to create/import an IC
identity. When the user selects this option, the browser shows the IC
identity manager where the user can create a master key (optional) and a
device key (required). A master key, once created, will be exported to
cold storage. After the identity setup is complete, the user interacts
with the canister using the new IC identity.

When a user, for the first time on a new device, interacts with a
canister hosted on the IC, the front page shows an option to
create/import an IC identity. When the user selects this option, the
browser shows the IC identity, where the user can import the master key
from cold storage, and create a new device key, authorizing the device.
After authorization is complete, the user interacts with the canister
using the IC identity.

When a user interacts with another canister hosted on the IC, from a
device that has been used with the IC before, the front page shows an
option to use the IC identity. When the user selects this option, the
browser shows the IC identity manager, where the user can authorize the
use of the identity for that canister. After the authorization is
complete, the user interacts with the canister using the IC identity.

When a user interacts with a canister on a device where they used the
same canister before, the front page shows an option to "sign in". When
the user selects this option, the user interacts with the canister using
the IC identity.

TODO: can you add something about the expected developer experience?

# Prior Art

## Technologies

### DER encoding

We use different types of public keys to support different technologies
on the client side, such as web authentication and web cryptography. We
use [DER](https://en.wikipedia.org/wiki/X.690#DER_encoding) (a subset of
ASN.1) to encode all public keys, so that the replica can unambiguously
determine the types of keys used for authenticating the request. More
concretely, we follow [RFC 8410](https://tools.ietf.org/html/rfc8410) to
encode public keys for Ed25519.

### Web authentication

[Web authentication (or WebAuthn)](https://www.w3.org/TR/webauthn/) is a
W3C standard for accessing FIDO tokens. For public-key-based
credentials, the private (signature) key is stored on a FIDO token and
is never exported. The web authentication API then allows a web
application to obtain a signature (from the FIDO token) on a given
challenge bit string. Web authentication is supported by all major
browsers (Chrome, Safari, Firefox, Edge) and by many consumer devices
that come with a biometric sensor (many laptops, tablets, and phones).

### Public-key certificates

Public-key certificates are widely used for delegation of authority,
such as in the [PKI](https://en.wikipedia.org/wiki/X.509). Effectively,
one uses a signature private key `sk-a` to sign a different public key
`pk-b` along with some metadata such as the expiration, and this
*certificate* is then presented (to a verifying party) by the owner of
private key `sk-b` (which corresponds to the `pk-b`). That way, the
owner of `sk-a` can delegate certain authorities to the owner of `sk-b`
(the verification checks the certificate using `pk-a`).

### Cold-storage keys

[BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
describes a format for storing private keys via a mnemonic code. BIP-39
is widely used in the area of cryptocurrencies, and most hardware
wallets use it for backup of the user’s private master key. The user
writes the mnenomic code (consisting of words in a language of their
choice) to a sheet of paper, which is stored at some secure place. The
key can be restored by entering the mnemonic code.

### Single sign-on

The web has solutions such as [OpenID
Connect](https://openid.net/connect/) or
[SAML](https://en.wikipedia.org/wiki/SAML_2.0). They are not immediately
useful for us, since they need a trusted identity provider for each
user, but the user flows (being redirected to the identity provider,
then back to the service provider) is already familiar to many users.

# Detailed Design

## Considered Solutions

### Public key encoding

The main alternative to DER is
[COSE](https://tools.ietf.org/html/rfc8152), which is based on CBOR
instead of ASN.1, and is used in web authentication. The use of CBOR is
the major advantage in COSE, since we use that format to encode the
requests. On the flip side, DER has better tooling support outside of
the IC, as the PKCS standards are based on it. It is also easier for us
to extend DER for new key formats by assigning new OIDs from our own
space, whereas extending COSE requires a change to the [type
registry](https://www.iana.org/assignments/cose/cose.xhtml) managed by
IANA.

### Private key storage

*Storing keys in browser local storage*. If keys are stored in plain in
browser local storage, the can be accessed by any code running using the
same origin. This can be prevented by using web cryptography to store
keys non-extractable. In both cases, keys could still be extracted by a
local malware attack. In addition, Safari’s policy to delete all state
of sites that have not been used in the last week means that user keys
are likely to be deleted.

*Storing keys in a OS key ring*. We could provide a browser extension
that stores keys in the OS key ring. While that circumvents the problems
of browser local storage, it would require us to develop a browser
extension/plugin for each major browser and OS, which seems infeasible
in the given time frame.

### Delegation

Besides certificates, there are two other major ways to implement
delegation between public keys.

*Storing keys in canister system storage*. For each canister and user
identity, a list of authorized public keys is stored in the affected
canister’s system state. When a user sends a request signed with some
public key and claiming some identity, the IC checks whether that public
key is authorized for that identity. **Advantages**: Almost no changes
to request format and canister code required. **Disadvantages**: Agent
needs additional API for key management (adding and removing), storage
is used by user but paid for by canister, which means canister needs API
to control the storage. That makes this solution overall more complex.

*Storing keys within canister memory*. There is no delegation in ICP,
but canisters implement key management on their own, likely relative to
a standardized API. **Advantages**: No changes to replica, no changes to
ICP. **Disadvantages**: Harder to consistently integrate with Motoko (at
least short term), needs support from canisters.

### Cold-storage (master) keys

Multiple alternatives have been discussed:

-   Deriving key from a password: While this is easy to use, a
    password-derived key does not contain sufficient entropy and is not
    considered secure.

-   Exporting the key to an encrypted PEM file: Already clunky on a
    desktop, unusable on mobile.

-   Hardware wallet: We cannot require each user to have one.

-   Server-based solution such as oblivious PRF or threshold sharing:
    Not realistic in the available time.

### Single sign-on

We previously discussed serving the identity manager in an iframe. This
solution has two main challenges:

-   Browsers get more aggressive in restricting what iframes can store.
    The iframe solution does not work in Safari and Brave, and it also
    does not with in Chrome and Firefox when 3rd party tracking is
    forbidden.

-   Using web authentication from an iframe is impossible.

## Recommended Solution

### Public-key encoding

All public keys are encoded in DER. That means:

-   Use of [RFC 8410](https://tools.ietf.org/html/rfc8410) for encoding
    Ed25519 keys.

-   Use of [DER-wrapped
    COSE](https://docs.dfinity.systems/dfinity/spec/public/index.html#signatures)
    for web authentication keys.

We need a unambiguous encoding of different types of public keys, and
DER suits out needs better than COSE.

### Private-key storage

Wherever possible, we use web authentication to store device keys. The
main reason is that web authentication allows us to keep the user’s
private key in secure hardware, where it cannot be extracted. As a
fallback mechanism, we keep the current solution of storing keys in
browser local storage.

Web authentication does not (yet) allow signing without user
consent/interaction. As that means the user would be required to
interact with their device for every query that is sent to the IC, we
use
[delegation](https://docs.dfinity.systems/dfinity/spec/public/index.html#authentication):
When loading the canister page first, the front end creates a standard
Ed25519 (or a web cryptography) key that it keeps in local storage, and
is used as a session key with a short expiration. It then creates a
delegation (certificate) from the web authentication key to the session
key. Queries are then signed with the session key, and also contain the
delegation.

### Cold-storage (master) keys

The master key is an Ed25519 key, the private key is exported and
imported as a mnemonic code via BIP-39. The master key is never *stored*
in the browser. When it is created, the front end shows the mneminoc
code to the user, creates a web authentication key for the device, and
delegates from the master key to the device web authentication key. When
authorizing a new device, the master key is imported, the device’s web
authentication key is created and authorized by the master key, and the
master key is again dropped from memory.

### Single sign-on

The identity manager uses a specific origin (e.g.
`identity.dfinity.network`). It is implemented as a full page, not an
iframe. The identity manager uses a web authentication key and
additionally keeps the following data in browser local storage:

-   Auxiliary information needed to access the web authentication key

-   (If available) Master public key (**not** private key)

-   (If available) Delegation from the master key to the web
    authentication key

-   List of front ends canister id authorized to use the identity, and
    for each front end the back end canister ids that may be accessed as
    well as a user-friendly free-text name of that front end

The main concept in the identity manager is that of a *delegation*. A
delegation here means that some canister front end (which is served by a
canister) is allowed to access **some** background canisters using as
sender of the requests the principal derived from the user’s master key
(or if that does not exist, the user’s web authentication key).
Delegations are scoped, meaning each canister front end may access only
specific back end canisters. For Mercury I, the identity manager has to
enforce the following rules:

-   Every canister front end may access only the canister serving the
    front end assets and at most **one** additional canister.

-   No canister may be accessed by two different front end canisters.
    (I.e. at any point in time, access to a canister is delegated to
    only **one** front end.)

The identity manager creates *delegation certificates* when asked to do
so by a canister front end that has an active delegation. A delegation
certificate contains the delegate public key (supplied by the canister
front end), the list of allowed targets (containing the id of the front
end canister itself and at most one additional canister id), an
expiration time (default 15 minutes unless configured otherwise by the
user), and a signature by the web authentication key.

**Initialization**: When the user opens the identity manager (whether by
visiting `identity.dfinity.network` directly or by being redirected from
some canister front wnd), the identity manager checks the browser local
storage for the above data to be available. If the local storage is
found to be empty, the following procedure is followed:

1.  The user is asked to (a) create a new master key, (b) import an
    existing master key, (c) proceed without master key. In cases (a)
    and (b), the respective part of the BIP-39 mechanism is used.

2.  Create new web authentication key, store auxiliary information in
    local storage

3.  If not (c), then after the master key is created and exported, or
    imported, then create delegation to the web authentication key.

**Direct visit**: If the user visits `identity.dfinity.network`
directly, the identity manager shows a list of acive delegations. The
user can view and delete the delegations. Optionally, the user may be
allowed to set the expiration time for delegation certificates, as well
as edit the free text name of that canister front end.

**Redirection**: When the user accesses a canister, and decides to use
the user’s IC identity, the browser is redirected to the identity
manager. First, the canister front end first generates a new (Ed25519 or
better web cryptography) session key pair. Second, the canister front
end redirects the browser to `identity.dfinity.network`, passing:

-   the public key

-   the suggested free text name for the front end

-   optionally the additional canister id that the front end wants to
    access

as parameters. The identity manager then proceeds as follows:

-   If no delegation for that canister front end exists, ask the user
    whether a new delegation shall be created. Let the user edit the
    free text name of the canister as well as the expiration. Create the
    new delegation (to the front end canister and, if provided, the
    additional canister requested by the front end) in the browser local
    storage.

-   Sign a delegation with the web authentication key toward the session
    public key passed by the canister front end, and redirect the
    browser back to the canister front end, passing along the
    delegations and the master public key (if present, otherwise web
    authentication key). The canister front end can then proceed using
    the session key.

## Public API

TODO: can you fill this in?

## Prototype

## Security Considerations

### Public-key encoding

Encoding the public key in DER will need an additional library in the
Javascript agent. No other security impact in the agent, as only the
encoding of public values is affected.

### Private-key storage

In terms of security, web authentication is preferable over our current
solution since the private key is stored in a secure hardware token
instead of on the user’s disk.

Session keys stored in the browser storage are (as long as we use
Ed25519) again extractable by the canister front end, including code
that may be injected due to a security flaw in the canister. While the
session key is time-restricted by a short expiration, it could still be
exfiltrated and leaked during its validity period. One countermeasure
that we should implement as soon as possible is web cryptography, and
storing the private key so it cannot be exported by Javascript.

### Cold-storage (master) key

The following risks apply with respect to the master key:

-   The randomness used for generating the master key must be secure.
    The key should use randomness from the `getRandomValues()` method of
    [web
    cryptography](https://www.w3.org/TR/WebCryptoAPI/#Crypto-method-getRandomValues)
    to ensure the randomness is good.

-   The user must keep their mnemonic code in a secret place. We should
    provide clear explanations.

-   The key will be in the browser memory. To protect the key:

    -   We ensure that the key is in memory only when it is used, and
        dropped immediately after the signing operation.

    -   The key is not stored permanently in the browser (no local
        storage, IndexedDB, …)

    -   The key is only imported in the identity manager, which for
        Mercury is served from the Foundation-provided bootstrap server.
        (The bootstrap server anyway has to be trusted in the web front
        end.)

When the user delegates from the master key to any one canister front
end, that canister front end can send requests to any canister on the
IC. That way, a malicious canister front end can attack the user. This
is planned to be resolved by restricting delegations to specific lists
of canisters, as outlined in this [draft
PR](https://github.com/dfinity-lab/ic-ref/pull/212).

### Single sign-on

One main security consideration for the identity manager affects how it
is served. As we initially serve it in the same way as we serve the
bootstrap, we do not add an additional trust assumption.

The second main consideration relates to cross-canister requests. If
delegations for the same master key to different canisters are not
scoped, then a potentially malicious or vulnerable canister front end
can access any canister in the name of the user. The main security
mechanism here is *scoping*: The user explicitly restricts the validity
of the delegation. As we do not want to bother the user with checking
canister ids (actually, there is no good way for a user to do those
checks), we restrict the mechanism to explicitly keep the access of
different front ends separate. That way, we can mostly exclude that a
malicious or vulnerable front end accesses precious user data in other
canisters. This mechanism will have to be improved in the future, to
allow for controlled access sharing between different canister front
ends.

## Performance Considerations

Encoding the public key in DER can be implemented as a one-time
operation. The encoding requires additional 12 bytes for Ed25519, and
additional 19 bytes for web authentication keys.

Using web authentication for signing does not significantly impact
performance on the computational side; the most significant impact comes
from the user providing consent by interacting with the device. Web
authentication signatures are larger than plain ones (for my example
that is 238 bytes vs. 64 bytes).

Delegations do not significantly affect the performance in the agent, as
this only incurs one additional signature per message. A delegation will
add around 150 bytes for the session key. Overall, that means a web
authentication request will be about 350 bytes longer than a transaction
signed via plain Ed25519.

Handling the master public key is a sufficiently infrequent operation.
Passing an additional delegation in every request adds around 150 bytes
to each request using that delegation.

The identity manager is used whenever the user starts a new session. The
operation itself is not expensive, but this does introduce latency in
the process mostly due to redirecting the browser.

# Breaking Changes

Switching the encoding of Ed25519 keys to DER is a breaking change (or
rather: switching off accepting raw Ed25519 keys is a breaking change).
We cannot deactivate raw Ed25519 keys during Sodium, as the principals
sent to us by canister owners use that type of key. Therefore, we will
deprecate raw Ed25519 keys during the switch from Sodium to Mercury.

## Deprecation

# Documentation

The interaction between agent and IC, including the exact formats, is
documented in the [public
spec](https://docs.dfinity.systems/dfinity/spec/public/index.html).

TODO: how will we document the new authentication library for
developers?

TODO: We certainly have to put up documentation explaining the master
key and the identity manager to end users.

# Lifecycle

## Integration Plan

The DER-encoding of Ed25519 keys is a minor change and can be
implemented after the last SDK version for Sodium and before the first
SDK version for Mercury is launched. The encoding also has to be
supported in `dfx` (there is a working implementation in Eric’s branch).

Web authentication is an additional feature that only needs to be
supported in the browser agent.

## Publishing Plan

## Rollout / Migration

As switching the encoding of Ed25519 keys is a breaking change for users
(at least those that have a principal in Sodium), roll out of the change
removing raw Ed25519 can only be done when switching from Sodium to
Mercury I.

Web authentication as an additional feature can be rolled out whenever
ready.

TODO: For identity manager.

## Rollback Plan

For DER-encoded Ed25519, if switching is impossible, then we can keep
the current heuristic for decoding raw Ed25519 keys in the replica (for
some further time).

If the new web authentication features do not work, we hold back on
rolling them out.

## Maintenance Plan

TODO: Any particular plan needed for the public key encoding or the web
authentication part?

TODO: Definitely needed for the identity manager

# Work Breakdown

See [the project board](https://github.com/orgs/dfinity/projects/4).
