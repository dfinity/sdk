# Overview

The purpose of this document is to describe the management of
cryptographic keys in the browser user agent. The document complements
the ones on the general architecture of the user agent (which describes
how the agent is served to the browser) and the public specification of
the Internet Computer (which describes the formats of the expected
inputs and provided outputs of the IC).

The user manages multiple user keys on the browser while mitigating the
attack surface area. In the end, a user should be able to utilize
multiple browsers to authenticate against a canister.

# Contributions

We answer the following questions here:

1.  Acknowledging that browser storage lacks the persistent trait and
    backup, how does a user recover their keys?

2.  What is our browser support story?

Questions raised:

1.  What is our UX/UI for authenticating on a new browser?

2.  Do we want to support multiple users/individuals/profiles on a
    browser?

Starting to clarify, but contingent on other points

1.  Our browser security model

2.  Do we need key derivation

3.  Synchronization Means

## Problem Statement

We want to minimize the risk of key leakage (partial or total) when one
uses the browser to issue requests to the Internet Computer via the
Userlibrary.

## Requirements

## Related Art

-   WebCrypto API Spec

Specifies an interface for secure implementation of limited and simple
cryptographic operations. The correctness and security of the
implementation is the responsibility of each browser. Chromium and
Firefox support the spec, with Safari, Edge and IE trailing. Note that
the performance of each operation is also not specified.

<https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API>

-   JSON Web Key (JWK)

<https://tools.ietf.org/html/rfc7517>  Example

    {
       "keys": [
            {
                "kty": "EC",
                "d": "UcfnEr0vwuK5iptuX6LE6OAc9amRiNPVMOpWVl7v6rk",
                "use": "sig",
                "crv": "P-256",
                "x": "1SPWydvsUp70xHJGOOJ8Y5w6uhoEPP_nnRnQorGHdbw",
                "y": "Wf1lIoSwfNBAxvqEDm8GtCh2Tb480ktXp0R8EyEwd4U",
                "alg": "ES256"
            }
        ]
    }

## Browser Compatibility

-   Edge: Despite the negative statements in the documentation on
    Microsoft’s web page, the newer versions of Edge appear to support
    the necessary parts of the WebCrypto standard.

-   Firefox: The WebCryptoSubtle `exportKey` method is broken for
    parameter `pkcs8`, which is supposed to output a DER encoding of the
    key. We use DER encoding when referencing the key in ingress
    messages. DER turns out to be an ASN.1 encoding of the raw key
    exported with parameter `raw`, so we can work around this issue for
    public keys. For private keys, where compatibility with non-browser
    software is less critical, we can use a format such as JWK instead.
    The bug in Firefox has been open for several years.

-   Internet Explorer: While WebCrypto functionality is implemented, the
    interface of the methods differs from the standard. The methods
    return a `CryptoOperation` instead of a `Promise`.

# Expected User/Developer Experience

In a first step, the user interaction is supposed to remain the same.
(Only the implementation changes in order to adapt to the current
version of the public specification, and to use the cryptographic
algorithms supported by the WebCrypto API.)

In a second step, the agent allows the user to export/import a private
key. The reason for this is that key storage in the browser local
storage can hardly be considered persistent.

In a third step, the agent will allow the client to link multiple
devices. Subsequently, we will allow elevated security levels with more
secure storage of cryptographic private keys.

# Detailed Design

## Considered Solutions

We describe alternative solutions that were discarded in favor of the
recommended solution described below.

### Key derivation

One approach is to utilize a main key for each device; then derive a new
key from the main key, per canister. When we authorize a new device we
provide a certificate for the new device, and each key on the new device
is derived. However, a key derivation scheme does not offer us any
advantages related to revocation and handling, but would require a
non-WebCrypto implementation of cryptographic algorithms.

### One-key per canister

The approach still uses a main key per device with different keys per
canister. In contrast to the above approach, however, the keys for
individual canisters are not derived from the main key, but generated
according to the usual procedures. This involves more complex handling
of keys, and eventually more flexible revocation, but the advantages are
initially rather small.

## Recommended Solution

We use WebCrypto to generate and store keys, and to sign messages. Note
that we thus entrust the correctness and security of the implementation
to the browser that realizes the WebCrypto API specification. We
consider this an acceptable choice given the status quo in the browser
scene.

### One key per device, securely managed by the agent

Each device (i.e. browser) has a single private cryptographic key. This
key is managed in the user agent and stored in IndexedDB, separated from
the canister front ends by origin. (Requests from the front end are
passed to the secured user agent in a way that the target canister is
still visible, so that cross-canister attacks can be avoided.)

### Key pair generation

On Userlib load:

1.  Check browser version:

    1.  If Edge (&gt;=79 ) // The browser is Chromium based

    2.  || Chrome

    3.  || Firefox

    4.  || IE () // Not sure oldest version supported here

    5.  || Safari

    6.  continue

    7.  else

    8.  Warn "WebCrypto API possibly not supported" // The problem here
        is that even if the browser supports it we can not // say
        anything about the implementation or its performance.

On makeAuthTransform:

1.  Let `canister-id` be the canister id stated in the request. Check
    that the `postMessage` invoking the request comes from origin
    `canister-id.ic.org`. If not, then abort.

2.  Open connection to IndexedDB

3.  Check if browser supports generateKey, sign and importKey for ECDSA
    P256

4.  If not fallback with a warning message to tweetnacl (key now stored
    in indexeddb)

5.  create key if none found (as exportable) // This seems an
    inefficiency of IndexedDB and browser mentality — there is no way to
    backup IndexedDB

6.  load key // a bit paranoid here, but IndexedDB is asynchronous; we
    need to at least check the key has been stored

7.  sign request

### Private key export / import

As the browser local storage (including IndexedDB) cannot reasonably
considered as persistent, we need to allow users to export their private
keys, and re-import it later. That way, users can backup their keys or
even switch to a different browser. The best level of compatibility
between different browsers is achieved using JWK format. (Firefox
fumbles on PKCS formats, all other browsers seem to follow standards.)

More technically, the export occurs through
`crypto.subtle.exportKey("jwk", keyPair.privateKey)`. They exported key
can then, e.g., be presented to the user as download.

In the future, a more user-friendly option seems to be to export the key
to a cloud service of the user’s choice. (This, of course, has to be
supported by the agent.) For this purpose, we may want to allow the user
to password-encrypt the exported key, which is achieved as follows (this
is pseudocode, but informed by the WebCrypto API):

    pbkfs2params = { name = "PBKDF2", hash = "SHA-256", salt = randomSalt, iterations = /* to be determined */ }
    aesKeyGenParams = { name = "AES-GCM", length = 128 }

    wrappingKey = crypto.subtle.deriveKey(pbkdf2Params, password, aesKeyGenParams, false, "wrapKey");

    gcmIv = /* BufferSource with EXACTLY 96 bits randomness */
    gcmParams = { name = "AES-GCM", gcmIv, additionalData = /* empty BufferSource */, tagLength = 128 }
    ciphertext = crypto.subtle.wrapKey("jwk", keyPair.privateKey, wrappingKey, gcmParams);

    store the object { randomSalt, gcmIv, ciphertext }

Exact parameter choices subject to change!

## User Profiles

Question:

Is this something we desire? Do we expect more than a single user to
access a browser? Right now a user would have to erase their history and
ensure the IndexedDB is erased to achieve this result.

Answer:

In the long run we should consider it, but not a feature for launch.

## Public API

## User Interaction & Authorization

Consider two devices "Alice" and "Bob". User wishes to access canisters
on both devices assuming the same corresponding principals. Each agent
on each device must:

1.  Know the canisters shared

2.  Principals to assume per canister

3.  Have a key to claim that principal

(One approach would be for the user to utilize a third party service
that provides secure key synchronization across devices.)

### Authorization Mechanism

#### Phase I:

As a first step, Alice explicitly adds the public key of Bob in the
target canister. For more information related to the interface please
see [Public Spec PR 26](https://github.com/dfinity-lab/ic-ref/pull/26).

#### Phase II:

The underlying authorization of a new key necessitates issuing a
certificate to Bob. When accessing a new canister Bob shall use that
certificate to authorize the corresponding key.

The certificate has the following structure:

      Certificate {
        alice_public_key: IssuerPublicKey,
        bob_public_key: AuthorizedPublicKey,
        expiration_utc_time: TimeAndDate,
        can_authorize(True): bool,
        alice_signature: Signature,
      }

Thus, an add\_key initial request to a canister must include:

1.  Certificate issued to Bob by Alice

2.  Certificate by Bob’s root key for the generated canister key

### UX above Authorization

Principal Stakeholder/Designer:

In this section we briefly discuss about how to exchange certificate
signing requests and certificates themselves between the two devices.

We break down the process as follows:

1.  Both devices need to exchange public keys in a trusted manner

2.  One (or both) devices need to exchange generated certificates.

    -   Example Approach

For the latter step we could use a public-key encryption scheme to share
the resulting certificate(s). We can achieve this with one of the
following approaches:

1.  Over Bluetooth with prompt on both devices and challenge requiring
    user input

2.  Alice providing a QR code (or a uri) that is scanned by Bob; then
    Bob provides a similar URI. User input is provided to verify
    authorization. (An extra scan is necessary if we require both
    devices to be authorized by the other.)

### Synchronization Mechanism

We can use a canister in the internet computer at the expense of making
user interactions extremely easier to access by the public, or provide
access to a third party service (such as by Google, Apple, Dropbox) that
will act as a provider also. We do not address this point here
explicitly.

## Prototype

Code:

-   Check

<!-- -->

    if (!window.crypto || !window.crypto.subtle) { alert("Browser does not support a secure framework."); }

-   Generate Key

<!-- -->

    const getPublicKey3 = async () => {

     const options = { name: 'ECDSA', namedCurve: "P-256", };
    const keys = await window.crypto.subtle.generateKey( options, false,
     ['sign', 'verify'], );
    // Store keys in Indexdb

    // This is not going to be as easy it seems however, because Firefox
    // is not supporting public key export for pkcs8 container format.
     const publicKey = await window.crypto.subtle.exportKey('pkcs8', keys.publicKey);


     let body = window.btoa(String.fromCharCode(...new Uint8Array(publicKey)));
     body = body.match(/.{1,64}/g).join('\n');
     return `-----BEGIN PUBLIC KEY-----\n${keys.publicKey}\n-----END PUBLIC KEY-----`;
    };

## Security Considerations

This is a preliminary security model for the browser. We assume user
library acts honestly; the adversary can not corrupt it. Requests and
scripts can be run across origins.

RequestId computation, signing of the ingress message need to happen in
the secure origin. This is to ensure that we attempt delivery of a
correctly signed message to the corresponding canister. It is also
prudent to have sending in the same origin, though it should not affect
security.

## Persistence Considerations

Recall that WebCrypto API enforces that one can not parse the secret key
even in the same origin. Then one major consideration of using the
WebCrypto API is persistence and restoration of the value.

The WebCrypto API supports an importKey operation, usually using JWK.
IndexedDB is the suggested means of "persisting" values. Note, however,
that IndexedDB as part of a browser’s localstorage is more ephemeral in
nature and acts as a long-term user cache.

## Performance Considerations

One key consideration is that WebCrypto is an API specification, that is
supported by the latest versions of browsers. However, the specification
inherently does not specify performance characteristics. In this design
we only consider signing interfaces and latest major browser releases.

# Breaking Changes

N/A

## Deprecation

The current auth API of the userlibrary will be modified to be
asynchronous in nature.

# Documentation

Documentations is necessary when the whole authentication flow for
browsers is complete.

# Lifecycle

## Integration Plan

N/A for now

In the future, we might want to enable similar operations in dfx.

## Publishing Plan

N/A

## Rollout / Migration

N/A

## Rollback Plan

As initially we introduce no user facing changes, nothing changes from a
user’s perspective until a user interface for authorization and key
loading is introduced. We can rollback to previous version with little
issue. Keys are currently thought disposable. As we will be using a
different storage layer falling back to old code will simply assume a
key was never generated.

## Maintenance Plan

# Work Breakdown

1.  Use IndexedDB for keys & switch keys to use JWK format

2.  Add check for WebCrypto API support and warnings (can’t be tested
    with current setup reliably)

3.  Add WebCrypto API in makeAuthTransform

4.  Design and facilitate a UX/UI for key authorization

5.  Figure out a way to test (contingent on testing framework at the
    time)

6.  Implement the decided solution for key authorization
