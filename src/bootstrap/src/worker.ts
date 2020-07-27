import { ProxyMessageKind, ProxyStubAgent } from '@dfinity/agent';
import { createAgent } from './host';
import { SiteInfo } from './site';

async function bootstrap() {
  const agent = await createAgent(await SiteInfo.worker());
  (window as any).ic = {
    agent,
  };
  const stub = new ProxyStubAgent(msg => {
    switch (msg.type) {
      case ProxyMessageKind.CallResponse:
        const response = msg.response.response;
        msg.response.response = JSON.parse(JSON.stringify(response));
    }
    window.parent.postMessage(msg, '*');
  }, agent);

  window.addEventListener('message', ev => {
    stub.onmessage(ev.data);
  });

  // Ping the server, and if it works send our ACK message to the parent.
  // If it doesn't work because of a 401 UNAUTHORIZED code, send a login
  // message to tell the parent we need to login.
  agent
    .status()
    .then(_json => {
      window.parent.postMessage('ready', '*');
    })
    .catch((error: Error) => {
      if (error.message.includes('Code: 401')) {
        window.parent.postMessage('login', '*');
      } else {
        throw error;
      }
    });
}

bootstrap().catch(error => {
  (console as any).error(error);
  window.parent.postMessage({ error }, '*');
});
