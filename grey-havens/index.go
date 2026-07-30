package main

const indexHTML = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Grey Havens — Life of a Packet</title>
  <style>
    :root { color-scheme: dark; --ink:#e8ede8; --muted:#9daaa3; --sea:#65c7c3; --panel:#142522; --line:#29433e; }
    * { box-sizing: border-box; }
    body { margin:0; min-height:100vh; color:var(--ink); background:radial-gradient(circle at 50% -20%,#315b55 0,#10201e 38%,#091311 75%); font:16px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace; }
    main { width:min(980px,calc(100% - 32px)); margin:0 auto; padding:72px 0; }
    header { margin-bottom:36px; } h1 { margin:0; font:clamp(2.5rem,8vw,5.5rem)/.95 Georgia,serif; letter-spacing:-.04em; }
    header p { max-width:650px; color:var(--muted); font-family:system-ui,sans-serif; }
    h2 { margin:42px 0 8px; font:2rem/1.1 Georgia,serif; } .intro { margin:0 0 18px; color:var(--muted); font-family:system-ui,sans-serif; }
    .osi { display:grid; grid-template-columns:repeat(7,1fr); gap:6px; margin:22px 0 34px; }
    .osi-layer { position:relative; min-height:155px; padding:13px 11px; border:1px solid var(--line); border-radius:9px; background:rgba(20,37,34,.64); }
    .osi-number { display:block; color:var(--sea); font-size:12px; } .osi-layer strong { display:block; margin:2px 0 9px; font-size:13px; }
    .osi-layer p { margin:0; color:var(--muted); font:12px/1.4 system-ui,sans-serif; }
    .osi-layer small { position:absolute; bottom:10px; left:11px; right:11px; color:#d8e4df; font-size:10px; }
    .osi-note { padding:14px; border-left:3px solid var(--sea); color:var(--muted); background:rgba(20,37,34,.5); font:14px/1.5 system-ui,sans-serif; }
    form { display:flex; gap:10px; padding:10px; border:1px solid var(--line); border-radius:12px; background:rgba(20,37,34,.85); }
    input { flex:1; min-width:0; border:0; outline:0; padding:13px; color:var(--ink); background:transparent; font:inherit; }
    button { border:0; border-radius:8px; padding:0 20px; color:#061311; background:var(--sea); font:700 14px inherit; cursor:pointer; }
    button:disabled { opacity:.5; cursor:wait; }
    #summary { margin:28px 0 14px; color:var(--muted); }
    ol { list-style:none; margin:0; padding:0; }
    li { display:grid; grid-template-columns:88px 1fr; gap:18px; padding:24px 18px; margin-bottom:10px; border:1px solid var(--line); border-radius:12px; background:rgba(20,37,34,.64); animation:arrive .35s both; }
    .time { color:var(--sea); padding-top:3px; } .step-head { display:flex; gap:10px; align-items:center; flex-wrap:wrap; }
    .step-head strong { font-size:18px; } .layer { padding:2px 8px; border:1px solid var(--line); border-radius:999px; color:var(--sea); font-size:12px; }
    .explanation { margin:8px 0 12px; color:var(--muted); font-family:system-ui,sans-serif; }
    .detail { padding:8px 10px; border-radius:6px; color:#d8e4df; background:#091512; overflow-wrap:anywhere; }
    dl { display:grid; grid-template-columns:minmax(120px,180px) 1fr; gap:5px 14px; margin:12px 0 0; font-size:13px; }
    dt { color:var(--muted); } dd { margin:0; overflow-wrap:anywhere; }
    .error { color:#ff9e91; } @keyframes arrive { from { opacity:0; transform:translateY(5px); } }
    @media(max-width:800px) { .osi{grid-template-columns:repeat(2,1fr)} }
    @media(max-width:600px) { main{padding-top:40px} form{display:block} button{width:100%;height:45px} li{grid-template-columns:1fr}.time{padding:0} dl{grid-template-columns:1fr} dd{margin-bottom:5px} .osi{grid-template-columns:1fr}.osi-layer{min-height:110px} }
  </style>
</head>
<body><main>
  <header><h1>Grey Havens</h1><p>Follow one HTTP request from a name in your browser to bytes returning across the network.</p></header>
  <section aria-labelledby="osi-title"><h2 id="osi-title">The OSI model</h2><p class="intro">Seven layers describe how a message becomes a physical signal and returns as useful application data. Follow this example from Layer 7 downward when sending, then back upward when receiving.</p>
    <div class="osi">
      <article class="osi-layer"><span class="osi-number">Layer 7</span><strong>Application</strong><p>What programs say to each other.</p><small>HTTP request, DNS query</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 6</span><strong>Presentation</strong><p>Transforms, encrypts, or encodes data.</p><small>TLS encryption</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 5</span><strong>Session</strong><p>Maintains a conversation between endpoints.</p><small>TLS/HTTP session state</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 4</span><strong>Transport</strong><p>Reliable delivery between application ports.</p><small>TCP segments, ports 443/80</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 3</span><strong>Network</strong><p>Routes packets between different networks.</p><small>IP packet, source → destination</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 2</span><strong>Data link</strong><p>Moves frames across the local network hop.</p><small>Ethernet/Wi-Fi frame, MAC</small></article>
      <article class="osi-layer"><span class="osi-number">Layer 1</span><strong>Physical</strong><p>Transmits bits as electrical, light, or radio signals.</p><small>Copper, fibre, or Wi-Fi radio</small></article>
    </div>
    <p class="osi-note"><strong>Example:</strong> “GET /” starts as HTTP data (7), is encrypted by TLS (6), carried in TCP segments (4), wrapped in IP packets (3), placed in Ethernet or Wi-Fi frames (2), and transmitted as signals (1). Layer 5 is less distinct in the modern Internet stack; session behavior is usually handled by TLS and HTTP. Grey Havens currently observes Go's Layers 4, 6, and 7 directly. Layers 1–3 still happen, but require packet capture or interface data to inspect honestly.</p>
  </section>
  <h2>Trace a real request</h2><p class="intro">Every observed step below is labelled with the OSI layer it belongs to.</p>
  <form id="trace-form"><input id="url" name="url" value="example.com" aria-label="URL to trace" placeholder="https://example.com" required><button>Trace voyage</button></form>
  <div id="summary">Enter a destination to begin.</div><ol id="events"></ol>
</main><script>
const form=document.querySelector('#trace-form'), input=document.querySelector('#url'), button=form.querySelector('button'), summary=document.querySelector('#summary'), list=document.querySelector('#events');
form.addEventListener('submit',async(e)=>{e.preventDefault();button.disabled=true;button.textContent='Tracing…';summary.textContent='The packet has left the harbour…';list.replaceChildren();
  try { const response=await fetch('/api/trace',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({url:input.value})}); const data=await response.json(); if(!response.ok) throw new Error(data.error);
    summary.innerHTML=data.error?'<span class="error">'+escapeHTML(data.error)+'</span>':escapeHTML(data.status+' · '+data.protocol+' · '+data.total_ms+'ms · '+data.bytes_read+' bytes');
    data.events.forEach((event,index)=>{const row=document.createElement('li');row.style.animationDelay=(index*45)+'ms';const facts=(event.facts||[]).map(fact=>'<dt>'+escapeHTML(fact.label)+'</dt><dd>'+escapeHTML(fact.value)+'</dd>').join('');row.innerHTML='<span class="time">+'+event.at_ms+'ms'+(event.duration?'<br><small>'+event.duration+'ms elapsed</small>':'')+'</span><div><div class="step-head"><strong>'+escapeHTML(event.name)+'</strong><span class="layer">OSI '+event.osi_layer+' · '+escapeHTML(event.osi_name)+'</span><span class="layer">'+escapeHTML(event.protocol)+'</span></div><p class="explanation">'+escapeHTML(event.explanation)+'</p>'+(event.detail?'<div class="detail">'+escapeHTML(event.detail)+'</div>':'')+(facts?'<dl>'+facts+'</dl>':'')+'</div>';list.append(row)});
  } catch(error) { summary.innerHTML='<span class="error">'+escapeHTML(error.message)+'</span>'; } finally { button.disabled=false;button.textContent='Trace voyage'; }
});
function escapeHTML(value){const node=document.createElement('div');node.textContent=String(value);return node.innerHTML}
</script></body></html>`
