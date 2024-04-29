import React, { useEffect, useMemo, useState } from 'react';
import Container from 'react-bootstrap/Container';
import Row from 'react-bootstrap/Row';
import Button from 'react-bootstrap/Button';
// import Onboard from '@web3-onboard/core'
import { init, useConnectWallet } from '@web3-onboard/react'
import walletConnectModule, {
  // WalletConnectOptions,
} from "@web3-onboard/walletconnect";
import injectedModule from '@web3-onboard/injected-wallets'
import { ethers } from 'ethers'
import { Helmet } from 'react-helmet';
// import 'bootstrap/dist/css/bootstrap.min.css';
import { idlFactory as mainIdlFactory } from '../../../../declarations/main';
import { idlFactory as canDBIndexIdl } from '../../../../declarations/CanDBIndex';
import { idlFactory as personhoodIdl } from '../../../../declarations/personhood';
// import { Personhood } from '../../../../declarations/personhood/personhood.did'; // FIXME
import config from '../../config.json';
import ourCanisters from '../../our-canisters.json';
import { Actor, Agent, HttpAgent } from '@dfinity/agent';
import { ClipLoader } from 'react-spinners';
import { AuthContext } from '../auth/use-auth-client';

const walletConnectOptions/*: WalletConnectOptions*/ = {
  projectId:
    (config.WALLET_CONNECT_PROJECT_ID as string) ||
    "default-project-id",
  dappUrl: config.DAPP_URL,
};
 
const blockNativeApiKey = config.BLOCKNATIVE_KEY as string;

const onBoardExploreUrl = undefined;

const walletConnect = walletConnectModule(walletConnectOptions);
const injected = injectedModule()
const wallets = [injected, walletConnect]

const chains = [
  {
    id: 1,
    token: 'ETH',
    label: 'Ethereum Mainnet',
    rpcUrl: config.MAINNET_RPC,
  },
];

const appMetadata = {
  name: 'Example Identity App',
  icon: '/logo.svg',
  logo: '/logo.svg',
  description: 'Example app providing personhood on DFINITY Internet Computer',
  explore: onBoardExploreUrl,
  recommendedInjectedWallets: [
    { name: 'Coinbase', url: 'https://wallet.coinbase.com/' },
    { name: 'MetaMask', url: 'https://metamask.io' }
  ],
};

const accountCenter = {
  desktop: {
    enabled: true,
  },
  mobile: {
    enabled: true,
    minimal: true,
  },
};

// const onboard = Onboard({
//   wallets,
//   chains,
//   appMetadata
// })

const onboard = init({
  appMetadata,
  apiKey: blockNativeApiKey,
  wallets,
  chains,
  accountCenter,
});

// UI actions:
// - connect: ask for signature, store the signature, try to retrieve, show retrieval status
// - recalculate: recalculate, show retrieval status
function Person() {
  return <>
    <AuthContext.Consumer>
      {({agent, isAuthenticated}) =>
        <PersonInner agent={agent} isAuthenticated={isAuthenticated}/>
      }
    </AuthContext.Consumer>
  </>;
}  

function PersonInner(props: {agent: Agent | undefined, isAuthenticated: Boolean}) {
  const [signature, setSignature] = useState<string>();
  const [message, setMessage] = useState<string>();
  const [nonce, setNonce] = useState<string>();
  const [address, setAddress] = useState<string>();
  const [score, setScore] = useState<number | 'didnt-read' | 'retrieved-none'>('didnt-read');
  const [obtainScoreLoading, setObtainScoreLoading] = useState(false);
  const [recalculateScoreLoading, setRecalculateScoreLoading] = useState(false);

  const [{ wallet, connecting }, connect, disconnect] = useConnectWallet();

  useEffect(() => {
    if (wallet) {
      const ethersProvider = new ethers.BrowserProvider(wallet!.provider, 'any'); // TODO: duplicate code
      // This does not work:
      // ethersProvider.on('accountsChanged', function (accounts) {
      //   setAddress(accounts[0]);
      // });
      ethersProvider.send('eth_requestAccounts', []).then((accounts) => {
        setAddress(accounts[0]);
      });      
    } else {
      setAddress(undefined);
    }
  }, [wallet]);

  useEffect(() => {
    if (props.agent !== undefined) {
      // const backend = createBackendActor(ourCanisters.CANISTER_ID_PERSONHOOD, {agent: props.agent}); // TODO: duplicate code

      const actor = Actor.createActor(canDBIndexIdl, {canisterId: process.env.CANISTER_ID_CANDBINDEX!, agent: props.agent});
      async function doIt() {
        const [flag, score] = await actor.sybilScore() as [boolean, number];
        console.log("SCORE:", score);
        setScore(score);
      }
      doIt().then(() => {});
    };
  }, [props.agent]);

  async function recalculateScore() {
    try {
      setRecalculateScoreLoading(true);
      let localMessage = message;
      let localNonce = nonce;
      const personhood: Personhood = Actor.createActor(personhoodIdl, {canisterId: ourCanisters.CANISTER_ID_PERSONHOOD!, agent: props.agent}); // TODO: duplicate code
      if (nonce === undefined) {
        const {message, nonce} = await personhood.getEthereumSigningMessage();
        localMessage = message;
        localNonce = nonce;
        setMessage(localMessage);
        setNonce(localNonce);
      }
      let localSignature = signature;
      if (signature === undefined) {
        const ethersProvider = new ethers.BrowserProvider(wallet!.provider, 'any'); // TODO: duplicate code
        const signer = await ethersProvider.getSigner();
        let signature = await signer.signMessage(localMessage!);
        localSignature = signature;
        setSignature(localSignature);
      }
      try {
        const result = await personhood.submitSignedEthereumAddressForScore({address: address!, signature: localSignature!, nonce: localNonce!});
        const j = JSON.parse(result);
        let score = j.score;
        setScore(/^\d+(\.\d+)?/.test(score) ? Number(score) : 'retrieved-none');
      }
      catch(e) {
        setScore('retrieved-none');
        alert(e);
        console.log(e);
      }
    }
    finally {
      setRecalculateScoreLoading(false);
    }
  }

  return (
    <div className="App">
      <Helmet>
        <title>Zon Social Media - verify your identity</title>
      </Helmet>
      <Container>
        <Row>
          <h1>Prove That You Are a Real Person</h1>
          <p>Each human is allowed to vote in our social network only once (no duplicate voters or bots).</p>
          <p>You prove your human identity by collecting several <q>stamps</q> at{' '}
            <a target='_blank' href="https://passport.gitcoin.co" rel="noreferrer">Gitcoin Passport site</a>.</p>
          <p>The current version of this app requires use of an Ethereum wallet that you need
            both in Gitcoin Passport and in this app. (So,{' '}
            you will need two wallets: DFINITY Internet Computer wallet and Ethereum wallet.){' '}
            You don't need to have any funds in your wallets to use this app (because you will use an Ethereum wallet{' '}
            only to sign a message for this app, not for any transactions).{' '}
            In the future we are going to add DFINITY Internet Computer support to Gitcoin Passport,{' '}
            to avoid the need to create an Ethereum wallet to verify personhood in this app.</p>
          <h2>Steps</h2>
          <ol>
            <li>Go to <a target='_blank' href="https://passport.gitcoin.co" rel="noreferrer">Gitcoin Passport</a>{' '}
              and prove your personhood. You need to collect several stamps with summary score {config.MINUMUM_ACCEPTED_SCORE} points or more.</li>
            <li>Return to this app and<br/>
              <Button disabled={connecting} onClick={() => (wallet ? disconnect(wallet) : connect())}>
                {connecting ? 'connecting' : wallet ? 'Disconnect Ethereum' : 'Connect Ethereum'}
              </Button>{' '}
              with the same wallet, as one you used for Gitcoin Password.<br/>
              Your wallet: {address ? <small>{address}</small> : 'not connected'}.
            </li>
            <li>If needed,<br/>
              <Button disabled={props.isAuthenticated !== true || !wallet || typeof score === 'number' && score >= config.MINUMUM_ACCEPTED_SCORE} onClick={recalculateScore}>
                Recalculate your identity score
              </Button>
              <ClipLoader loading={recalculateScoreLoading}/>{' '}
            </li>
          </ol>
          <p>Your identity score:{' '}
            {score === 'didnt-read' ? 'Click the above button to check.'
              : score === 'retrieved-none' ? 'Not yet calculated'
              : `${score} ${typeof score == 'number' && score >= config.MINUMUM_ACCEPTED_SCORE
              ? '(Congratulations: You\'ve been verified.)'
              : `(Sorry: It\'s <${config.MINUMUM_ACCEPTED_SCORE}, you are considered a bot.)`}`}
          </p>
        </Row>
      </Container>
    </div>
  );
}

export default Person;
