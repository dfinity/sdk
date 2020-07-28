import {
  Agent,
  HttpAgent,
  makeAuthTransform,
  makeNonceTransform,
  Principal,
  ProxyAgent,
  ProxyMessage,
} from '@dfinity/agent';
import { SiteInfo } from './site';

export async function createAgent(site: SiteInfo): Promise<Agent> {
  const workerHost = site.isUnknown() ? undefined : await site.getWorkerHost();
  const host = await site.getHost();

  if (!workerHost) {
    const keyPair = await site.getKeyPair();
    const principal = Principal.selfAuthenticating(keyPair.publicKey);
    const creds = await site.getLogin();
    const agent = new HttpAgent({
      host,
      principal,
      ...(creds && { credentials: { name: creds[0], password: creds[1] } }),
    });
    agent.addTransform(makeNonceTransform());
    agent.setAuthTransform(makeAuthTransform(keyPair));

    return agent;
  } else {
    return createWorkerAgent(site, workerHost, host);
  }
}

async function createWorkerAgent(site: SiteInfo, workerHost: string, host: string): Promise<Agent> {
  // Create the IFRAME.
  let messageQueue: ProxyMessage[] | null = [];
  let loaded = false;
  const agent = new ProxyAgent((msg: ProxyMessage) => {
    if (!loaded) {
      if (!messageQueue) {
        throw new Error('No Message Queue but need Queueing...');
      }
      messageQueue.push(msg);
    } else {
      iframeEl.contentWindow!.postMessage(msg, '*');
    }
  });

  const iframeEl = document.createElement('iframe');

  iframeEl.src = `${workerHost}/worker.html?${host ? 'host=' + encodeURIComponent(host) : ''}`;
  window.addEventListener('message', ev => {
    if (ev.origin === workerHost) {
      switch (ev.data) {
        case 'ready':
          const q = messageQueue?.splice(0, messageQueue.length) || [];
          for (const msg of q) {
            iframeEl.contentWindow!.postMessage(msg, workerHost);
          }

          loaded = true;
          messageQueue = null;
          break;

        case 'login':
          const url = new URL(workerHost);
          url.pathname = '/login.html';
          url.searchParams.append('redirect', '' + window.location);
          window.location.replace('' + url);
          break;

        default:
          if (typeof ev.data === 'object') {
            agent.onmessage(ev.data);
          } else {
            throw new Error('Invalid message from worker: ' + JSON.stringify(ev.data));
          }
      }
    }
  });

  document.head.append(iframeEl);
  return agent;
}
