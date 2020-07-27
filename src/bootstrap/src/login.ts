import { SiteInfo } from './site';

const loginEl = document.getElementById('login')!;
const usernameEl = document.getElementById('username')! as HTMLInputElement;
const passwordEl = document.getElementById('password')! as HTMLInputElement;

SiteInfo.fromWindow().then(site => {
  site.getLogin().then(login => {
    if (login) {
      const [user, pass] = login;
      usernameEl.value = user;
      passwordEl.value = pass;
    }
  });

  loginEl.addEventListener('click', async () => {
    await site.setLogin(usernameEl.value, passwordEl.value);

    const url = new URL(window.location + '');
    const redirect = url.searchParams.get('redirect');

    if (redirect) {
      // console.log(redirect);
      window.location.replace(redirect);
    } else {
      alert('Login credentials saved.');
    }
  });
});
