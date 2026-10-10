use crate::error::Error;
use crate::session::{Session, World};
use serde::Deserialize;

pub(crate) const EDITABLE: &str = "function(e){if(!e||e.nodeType!==1)return false;if(e.getAttribute('aria-readonly')==='true')return false;if(e.isContentEditable)return true;if(e.tagName==='TEXTAREA')return !e.readOnly;if(e.tagName!=='INPUT')return false;var ty=(e.getAttribute('type')||'text').toLowerCase();return ['checkbox','radio','button','submit','reset','image','file','hidden','range','color'].indexOf(ty)<0&&!e.readOnly;}";

pub(crate) const ENABLED: &str = "function(e){return !e.disabled&&e.getAttribute('aria-disabled')!=='true'&&!(e.closest&&e.closest('fieldset[disabled]'));}";

pub(crate) const COVER: &str = "globalThis.__charmCover";

pub(crate) const RESOLVER: &str = r##"(function(matchText,editable,enabled){
  function norm(t){return (t==null?'':String(t)).replace(/\s+/g,' ').trim();}
  function isVisible(el){
    if(!el||el.nodeType!==1)return false;
    var r=el.getBoundingClientRect();
    if(r.width<=0&&r.height<=0)return false;
    var s=getComputedStyle(el);
    if(s.visibility==='hidden'||s.display==='none'||s.opacity==='0')return false;
    for(var p=el.parentElement;p;p=p.parentElement){if(getComputedStyle(p).opacity==='0')return false;}
    return true;
  }
  function accName(el,role){
    var lb=el.getAttribute&&el.getAttribute('aria-labelledby');
    if(lb){var t=lb.split(/\s+/).map(function(id){var e=document.getElementById(id);return e?norm(e.textContent):'';}).filter(Boolean).join(' ');if(t)return norm(t);}
    var al=el.getAttribute&&el.getAttribute('aria-label');
    if(al)return norm(al);
    if(el.id){var esc=(window.CSS&&CSS.escape)?CSS.escape(el.id):el.id;var l=document.querySelector('label[for="'+esc+'"]');if(l)return norm(l.innerText||l.textContent);}
    var wrap=el.closest&&el.closest('label');if(wrap)return norm(wrap.innerText||wrap.textContent);
    if(el.tagName==='IMG'){var a=el.getAttribute('alt');if(a)return norm(a);}
    if(role===undefined)role=implicitRole(el);
    return (role&&NAME_FROM_CONTENT[role])?norm(el.innerText||el.textContent):'';
  }
  var INPUT_ROLE={text:'textbox',search:'searchbox',email:'textbox',tel:'textbox',url:'textbox',password:'textbox',number:'spinbutton',checkbox:'checkbox',radio:'radio',range:'slider',button:'button',submit:'button',reset:'button',image:'button'};
  var NAME_FROM_CONTENT={button:1,link:1,heading:1,option:1,radio:1,checkbox:1,switch:1,tab:1,menuitem:1,menuitemcheckbox:1,menuitemradio:1,treeitem:1,cell:1,gridcell:1,columnheader:1,rowheader:1,tooltip:1};
  var ROLE_CANDIDATES='[role],a,area,button,input,select,textarea,h1,h2,h3,h4,h5,h6,img,nav,main,header,footer,aside,ul,ol,li,dialog,table,form';
  function implicitRole(el){
    var explicit=el.getAttribute&&el.getAttribute('role');
    if(explicit)return explicit.trim().toLowerCase().split(/\s+/)[0];
    var tag=el.tagName.toLowerCase();
    if(tag==='a'||tag==='area')return el.hasAttribute('href')?'link':null;
    if(tag==='button')return 'button';
    if(tag==='input'){var ty=(el.getAttribute('type')||'text').toLowerCase();if(ty==='hidden')return null;return INPUT_ROLE[ty]||'textbox';}
    if(tag==='select')return (el.multiple||el.size>1)?'listbox':'combobox';
    if(tag==='textarea')return 'textbox';
    if(/^h[1-6]$/.test(tag))return 'heading';
    if(tag==='img')return el.getAttribute('alt')===''?null:'img';
    if(tag==='nav')return 'navigation';
    if(tag==='main')return 'main';
    if(tag==='header')return el.closest('article,aside,main,nav,section')?null:'banner';
    if(tag==='footer')return el.closest('article,aside,main,nav,section')?null:'contentinfo';
    if(tag==='aside')return 'complementary';
    if(tag==='ul'||tag==='ol')return 'list';
    if(tag==='li')return 'listitem';
    if(tag==='dialog')return 'dialog';
    if(tag==='table')return 'table';
    if(tag==='form')return 'form';
    return null;
  }
  function order(list){var o=[];list.forEach(function(e){if(o.indexOf(e)<0)o.push(e);});return o.sort(function(a,b){return a===b?0:(a.compareDocumentPosition(b)&Node.DOCUMENT_POSITION_FOLLOWING)?-1:1;});}
  function text(e){return norm(e.innerText==null?e.textContent:e.innerText);}
  function keep(e,q){
    if(q.hasText&&!matchText(text(e),q.hasText))return false;
    if(q.hasNotText&&matchText(text(e),q.hasNotText))return false;
    if(q.has&&!find(q.has,e).length)return false;
    if(q.hasNot&&find(q.hasNot,e).length)return false;
    return q.visible==null||isVisible(e)===q.visible;
  }
  function xpath(path,root){var p=(root!==document&&path.charAt(0)==='/')?'.'+path:path;var s=document.evaluate(p,root,null,XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,null);var o=[];for(var i=0;i<s.snapshotLength;i++)o.push(s.snapshotItem(i));return o;}
  function byRef(q,root){var m=globalThis[q.registry];var e=m&&m[q.id];return (e&&e.isConnected&&(root===document||(root!==e&&root.contains(e))))?[e]:[];}
  function pick(q,root){var l=find(q.base,root);var e=q.last?l[l.length-1]:l[q.index];return e?[e]:[];}
  function find(q,root){
    root=root||document;
    if(q.kind==='css')return Array.prototype.slice.call(root.querySelectorAll(q.css));
    if(q.kind==='xpath')return xpath(q.path,root);
    if(q.kind==='ref')return byRef(q,root);
    if(q.kind==='within')return order([].concat.apply([],find(q.outer,root).map(function(o){return find(q.inner,o);})));
    if(q.kind==='filter')return find(q.base,root).filter(function(e){return keep(e,q);});
    if(q.kind==='and'){var b=find(q.b,root);return find(q.a,root).filter(function(e){return b.indexOf(e)>=0;});}
    if(q.kind==='or')return order(find(q.a,root).concat(find(q.b,root)));
    if(q.kind==='nth')return pick(q,root);
    if(q.kind==='testid'){var o=[];root.querySelectorAll('[data-testid]').forEach(function(e){if(e.getAttribute('data-testid')===q.id&&isVisible(e))o.push(e);});return o;}
    if(q.kind==='text'){var out=[];root.querySelectorAll('*').forEach(function(e){if(!isVisible(e))return;var t=norm(e.innerText);if(!matchText(t,q.match))return;var kids=e.children;for(var i=0;i<kids.length;i++){if(isVisible(kids[i])&&matchText(norm(kids[i].innerText),q.match))return;}out.push(e);});return out;}
    if(q.kind==='label'){var seen=[];root.querySelectorAll('label').forEach(function(lab){var t=norm(lab.innerText||lab.textContent);if(!matchText(t,q.match))return;var ctrl=null;var f=lab.getAttribute('for');if(f)ctrl=document.getElementById(f);if(!ctrl)ctrl=lab.querySelector('input,select,textarea,button,[contenteditable]');if(ctrl&&isVisible(ctrl)&&seen.indexOf(ctrl)<0)seen.push(ctrl);});root.querySelectorAll('[aria-label],[aria-labelledby]').forEach(function(e){if(isVisible(e)&&matchText(accName(e),q.match)&&seen.indexOf(e)<0)seen.push(e);});return seen;}
    if(q.kind==='role'){var rr=[];root.querySelectorAll(ROLE_CANDIDATES).forEach(function(e){if(!isVisible(e))return;if(e.closest('[aria-hidden="true"]'))return;var ir=implicitRole(e);if(ir!==q.role)return;if(q.name!=null&&!matchText(accName(e,ir),q.name))return;rr.push(e);});return rr;}
    return [];
  }
  function blocker(el,need){
    if(!el||el.nodeType!==1)return 'no match';
    if(!el.isConnected)return 'detached';
    var s=getComputedStyle(el),r0=el.getBoundingClientRect();
    if(!((r0.width>0||r0.height>0)&&s.visibility!=='hidden'&&s.display!=='none'&&s.opacity!=='0'))return 'not visible';
    if(!enabled(el))return 'disabled';
    if(need==='editable'&&!editable(el))return 'not editable';
    return null;
  }
  function rect(el){var r=el.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}
  function cover(top){globalThis.__charmCover=top;var role=implicitRole(top)||top.tagName.toLowerCase();var nm=accName(top,role);return role+(nm?' '+JSON.stringify(nm):'');}
  function actionable(el,ms,need){
    var why=blocker(el,need);
    if(why)return Promise.resolve({ok:false,reason:why});
    var r0=el.getBoundingClientRect();
    var dw=el.ownerDocument.defaultView||window,dd=el.ownerDocument||document;
    if(r0.top<0||r0.left<0||r0.bottom>dw.innerHeight||r0.right>dw.innerWidth)el.scrollIntoView({block:'center',inline:'center'});
    if(need==='editable')return Promise.resolve({ok:true,box:rect(el)});
    function bx(){var r=el.getBoundingClientRect();return [r.x,r.y,r.width,r.height];}
    var a=bx();return new Promise(function(res){setTimeout(function(){res({ok:false,reason:'unstable'});},ms);dw.requestAnimationFrame(function(){var b=bx();var stable=Math.abs(a[0]-b[0])<0.5&&Math.abs(a[1]-b[1])<0.5&&Math.abs(a[2]-b[2])<0.5&&Math.abs(a[3]-b[3])<0.5;if(!stable){res({ok:false,reason:'unstable'});return;}var r=el.getBoundingClientRect();if(r.width<=0&&r.height<=0){res({ok:false,reason:'zero size'});return;}var cx=r.x+r.width/2,cy=r.y+r.height/2;if(cx<0||cy<0||cx>dw.innerWidth||cy>dw.innerHeight){res({ok:false,reason:'offscreen'});return;}var top=dd.elementFromPoint(cx,cy);var hit=!top||el===top||el.contains(top)||top.contains(el);if(!hit){res({ok:false,reason:'obscured',coveredBy:cover(top)});return;}res({ok:true,box:rect(el)});});});
  }
  return {find:find,actionable:actionable};
})"##;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    #[must_use]
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub enum Blocker {
    #[serde(rename = "no match")]
    NoMatch,
    #[serde(rename = "detached")]
    Detached,
    #[serde(rename = "not visible")]
    NotVisible,
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "not editable")]
    NotEditable,
    #[serde(rename = "unstable")]
    Unstable,
    #[serde(rename = "zero size")]
    ZeroSize,
    #[serde(rename = "offscreen")]
    Offscreen,
    #[serde(rename = "obscured")]
    Obscured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Need {
    Pointer,
    Editable,
}

impl Need {
    pub(crate) fn js(self) -> &'static str {
        match self {
            Self::Pointer => "pointer",
            Self::Editable => "editable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Actionable {
    pub ok: bool,
    #[serde(default)]
    pub reason: Option<Blocker>,
    #[serde(default, rename = "box")]
    pub bounds: Option<BoundingBox>,
    #[serde(default, rename = "coveredBy")]
    pub covered_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ClickResult {
    pub clicked: bool,
    pub reason: Option<Blocker>,
    pub covered_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Read<'a> {
    BoundingBox,
    Visible,
    InputValue,
    TextContent,
    InnerText,
    Attribute(&'a str),
    Enabled,
    Checked,
    Focused,
    Attached,
    Class,
    Css(&'a str),
    Id,
    JsProperty(&'a str),
    Editable,
    Empty,
    InViewport,
}

impl Read<'_> {
    pub(crate) fn expr(&self, element: &str) -> String {
        let body = self
            .body()
            .map_or_else(|| self.parameterized(), str::to_owned);
        format!("(()=>{{const e={element};{body}}})()")
    }

    fn body(&self) -> Option<&'static str> {
        Some(match self {
            Self::BoundingBox => {
                "if(!e)return null;const r=e.getBoundingClientRect();return (r.width<=0&&r.height<=0)?null:{x:r.x,y:r.y,width:r.width,height:r.height};"
            }
            Self::Visible => {
                "if(!e)return false;const r=e.getBoundingClientRect();const s=getComputedStyle(e);return r.width>0&&r.height>0&&s.visibility!=='hidden'&&s.display!=='none';"
            }
            Self::InputValue => "return e?(e.value??null):null;",
            Self::TextContent => "return e?(e.textContent??''):null;",
            Self::InnerText => "return e?(e.innerText??''):null;",
            Self::Checked => {
                "if(!e)return false;return !!e.checked||e.getAttribute('aria-checked')==='true';"
            }
            Self::Focused => "return !!e&&document.activeElement===e;",
            Self::Attached => "return !!e;",
            Self::Class => "return e?(e.getAttribute('class')??''):null;",
            Self::Id => "return e?e.id:null;",
            Self::Empty => {
                "if(!e)return false;if(e.tagName==='INPUT'||e.tagName==='TEXTAREA')return !e.value;return !(e.textContent||'').trim();"
            }
            Self::InViewport => {
                "if(!e)return false;const r=e.getBoundingClientRect();const w=e.ownerDocument.defaultView||window;return r.width>0&&r.height>0&&r.right>0&&r.bottom>0&&r.left<w.innerWidth&&r.top<w.innerHeight;"
            }
            Self::Attribute(_)
            | Self::Css(_)
            | Self::JsProperty(_)
            | Self::Enabled
            | Self::Editable => {
                return None;
            }
        })
    }

    fn parameterized(&self) -> String {
        match self {
            Self::Attribute(name) => format!(
                "return e?e.getAttribute({}):null;",
                crate::selector::quote(name)
            ),
            Self::Css(name) => format!(
                "return e?getComputedStyle(e).getPropertyValue({}):null;",
                crate::selector::quote(name)
            ),
            Self::JsProperty(name) => format!(
                "if(!e)return null;const v=e[{}];return v===undefined?null:v;",
                crate::selector::quote(name)
            ),
            Self::Enabled => format!("if(!e)return false;return ({ENABLED})(e);"),
            Self::Editable => {
                format!("if(!e)return false;return ({EDITABLE})(e)&&({ENABLED})(e);")
            }
            _ => String::new(),
        }
    }
}

pub(crate) const REGISTER_REF: &str = r"function(id,registry){var m=globalThis[registry]||(globalThis[registry]={});m[id]=this;return true;}";

pub(crate) const READABLE: &str = r##"(function(md,max){
  function cap(t){t=t.replace(/[ \t]+\n/g,'\n').replace(/\n{3,}/g,'\n\n').trim();return t.length>max?t.slice(0,max):t;}
  if(!md)return cap((document.body&&document.body.innerText)||'');
  var SKIP={SCRIPT:1,STYLE:1,NOSCRIPT:1,TEMPLATE:1,SVG:1,HEAD:1,INPUT:1,SELECT:1,TEXTAREA:1,BUTTON:1,IFRAME:1};
  var BLOCK={P:1,DIV:1,SECTION:1,ARTICLE:1,HEADER:1,FOOTER:1,MAIN:1,NAV:1,ASIDE:1,FORM:1,FIGURE:1,FIGCAPTION:1,DL:1,DT:1,DD:1,FIELDSET:1,DETAILS:1,SUMMARY:1};
  function hidden(e){var s=getComputedStyle(e);return s.display==='none'||s.visibility==='hidden';}
  function flat(e){return kids(e,0).replace(/\s+/g,' ').trim();}
  function kids(e,d){var o='';e.childNodes.forEach(function(c){o+=render(c,d);});return o;}
  function list(e,d){var o='\n',i=0,pad='  '.repeat(d);Array.prototype.forEach.call(e.children,function(li){if(li.tagName!=='LI'||hidden(li))return;i++;o+=pad+(e.tagName==='OL'?i+'. ':'- ')+kids(li,d+1).replace(/\n{2,}/g,'\n').trim()+'\n';});return o+'\n';}
  function table(e){var o='\n\n';Array.prototype.forEach.call(e.rows,function(r,i){var c=Array.prototype.map.call(r.cells,function(x){return flat(x).replace(/\|/g,'\\|');});o+='| '+c.join(' | ')+' |\n';if(i===0)o+='|'+c.map(function(){return ' --- ';}).join('|')+'|\n';});return o+'\n';}
  function render(n,d){
    if(n.nodeType===3)return n.textContent.replace(/\s+/g,' ');
    if(n.nodeType!==1)return '';
    var t=n.tagName;
    if(SKIP[t]||hidden(n))return '';
    var h=/^H([1-6])$/.exec(t);
    if(h)return '\n\n'+'#'.repeat(+h[1])+' '+flat(n)+'\n\n';
    if(t==='A'&&n.hasAttribute('href')){var a=flat(n);return a?'['+a+']('+n.href+')':'';}
    if(t==='PRE')return '\n\n```\n'+n.innerText.replace(/\n$/,'')+'\n```\n\n';
    if(t==='CODE')return '`'+n.textContent+'`';
    if(t==='STRONG'||t==='B'){var b=flat(n);return b?'**'+b+'**':'';}
    if(t==='EM'||t==='I'){var m=flat(n);return m?'_'+m+'_':'';}
    if(t==='BR')return '\n';
    if(t==='HR')return '\n\n---\n\n';
    if(t==='IMG'){var alt=n.getAttribute('alt');return alt?'!['+alt+']('+n.src+')':'';}
    if(t==='UL'||t==='OL')return list(n,d);
    if(t==='TABLE')return table(n);
    if(t==='BLOCKQUOTE')return '\n\n'+kids(n,d).trim().split('\n').map(function(l){return '> '+l;}).join('\n')+'\n\n';
    if(BLOCK[t])return '\n\n'+kids(n,d)+'\n\n';
    return kids(n,d);
  }
  return cap(document.body?render(document.body,0).split('\n').map(function(l){return l.replace(/ +$/,'').replace(/^ (?! )/,'');}).join('\n'):'');
})"##;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextFormat {
    Text,
    Markdown,
}

impl Session {
    pub async fn text(&mut self, format: TextFormat, max_chars: usize) -> Result<String, Error> {
        let markdown = format == TextFormat::Markdown;
        let expression = format!("({READABLE})({markdown},{max_chars})");
        let value = self.evaluate_in(None, World::Isolated, &expression).await?;
        Ok(serde_json::from_value(value)?)
    }
}
