import{o as e,c as s,n as t,g as r,u as a,d as n,_ as o}from"./entry.665c13f0.js";const i={__name:"nuxt-error-page",props:{error:Object},setup(i){const l=i,{error:u}=l;(u.stack||"").split("\n").splice(1).map((e=>({text:e.replace("webpack:/","").replace(".vue",".js").trim(),internal:e.includes("node_modules")&&!e.includes(".cache")||e.includes("internal")||e.includes("new Promise")}))).map((e=>`<span class="stack${e.internal?" internal":""}">${e.text}</span>`)).join("\n");const c=Number(u.statusCode||500),p=404===c,m=u.statusMessage??(p?"Page Not Found":"Internal Server Error"),d=u.message||u.toString(),_=n(p?()=>o((()=>import("./error-404.e5b496ff.js")),["./error-404.e5b496ff.js","./entry.665c13f0.js","./entry.68362dcf.css","./error-404.efbd2657.css"],import.meta.url).then((e=>e.default||e)):()=>o((()=>import("./error-500.49a8bb9c.js")),["./error-500.49a8bb9c.js","./entry.665c13f0.js","./entry.68362dcf.css","./error-500.92b94fae.css"],import.meta.url).then((e=>e.default||e)));return(n,o)=>(e(),s(a(_),t(r({statusCode:a(c),statusMessage:a(m),description:a(d),stack:a(void 0)})),null,16))}};export{i as default};
