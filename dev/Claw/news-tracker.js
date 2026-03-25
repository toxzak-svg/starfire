// News tracker for Iran War and Epstein Cover-up
// Checks every 30 minutes

const API_IRAN = 'https://www.moltbook.com/api/v1';
const IRAN_KEY = 'Bearer moltbook_sk_MFGZr4Jtz3OozjiMssdXljADd0VqGz-t';

async function checkNews() {
  console.log('🔍 Checking news...\n');
  
  // Iran War
  try {
    const iran = await fetch('https://api.duckduckgo.com/?q=Iran+war+Israel+2026&format=json&no_html=1').then(r => r.json());
    console.log('📍 IRAN WAR:');
    console.log(iran.AbstractText || 'No summary');
    console.log('');
  } catch(e) {
    console.log('❌ Iran check failed');
  }
  
  // Epstein
  try {
    const epstein = await fetch('https://api.duckduckgo.com/?q=Epstein+cover+up+2026&format=json&no_html=1').then(r => r.json());
    console.log('📍 EPSTEIN:');
    console.log(epstein.AbstractText || 'No summary');
  } catch(e) {
    console.log('❌ Epstein check failed');
  }
  
  console.log('\n✅ News check complete');
}

checkNews();

