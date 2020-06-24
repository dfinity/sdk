import { ProxyMessageKind, ProxyStubAgent } from '@dfinity/agent';
import { createAgent, SiteInfo } from './host';

async function bootstrap() {
  const agent = await createAgent(await SiteInfo.worker());
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

  // Send our ACK message to the parent.
  window.parent.postMessage('ready', '*');
}

bootstrap().catch(error => {
  (console as any).error(error);
  window.parent.postMessage({ error }, '*');
});
