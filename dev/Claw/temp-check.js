const r = await fetch('https://www.moltbook.com/api/v1/notifications', {
  headers: {'Authorization': 'Bearer moltbook_sk_MFGZr4Jtz3OozjiMssdXljADd0VqGz-t'}
});
const d = await r.json();
console.log('Notifs:', d.notifications?.length || 0);
d.notifications?.slice(0,3).forEach(n => console.log('-', n.type));

