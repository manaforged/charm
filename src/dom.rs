use serde::Deserialize;

pub(crate) const RESOLVER: &str = r##"(function(matchText){
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
  function find(q,root){
    root=root||document;
    if(q.kind==='testid'){var o=[];root.querySelectorAll('[data-testid]').forEach(function(e){if(e.getAttribute('data-testid')===q.id&&isVisible(e))o.push(e);});return o;}
    if(q.kind==='text'){var out=[];root.querySelectorAll('*').forEach(function(e){if(!isVisible(e))return;var t=norm(e.innerText);if(!matchText(t,q.match))return;var kids=e.children;for(var i=0;i<kids.length;i++){if(isVisible(kids[i])&&matchText(norm(kids[i].innerText),q.match))return;}out.push(e);});return out;}
    if(q.kind==='label'){var seen=[];root.querySelectorAll('label').forEach(function(lab){var t=norm(lab.innerText||lab.textContent);if(!matchText(t,q.match))return;var ctrl=null;var f=lab.getAttribute('for');if(f)ctrl=document.getElementById(f);if(!ctrl)ctrl=lab.querySelector('input,select,textarea,button,[contenteditable]');if(ctrl&&isVisible(ctrl)&&seen.indexOf(ctrl)<0)seen.push(ctrl);});root.querySelectorAll('[aria-label],[aria-labelledby]').forEach(function(e){if(isVisible(e)&&matchText(accName(e),q.match)&&seen.indexOf(e)<0)seen.push(e);});return seen;}
    if(q.kind==='role'){var rr=[];root.querySelectorAll(ROLE_CANDIDATES).forEach(function(e){if(!isVisible(e))return;if(e.closest('[aria-hidden="true"]'))return;var ir=implicitRole(e);if(ir!==q.role)return;if(q.name!=null&&!matchText(accName(e,ir),q.name))return;rr.push(e);});return rr;}
    return [];
  }
  function blocker(el){
    if(!el||el.nodeType!==1)return 'no match';
    if(!document.contains(el))return 'detached';
    var s=getComputedStyle(el),r0=el.getBoundingClientRect();
    if(!((r0.width>0||r0.height>0)&&s.visibility!=='hidden'&&s.display!=='none'&&s.opacity!=='0'))return 'not visible';
    if(el.disabled||el.getAttribute('aria-disabled')==='true'||(el.closest&&el.closest('fieldset[disabled]')))return 'disabled';
    return null;
  }
  function actionable(el){
    var why=blocker(el);
    if(why)return Promise.resolve({ok:false,reason:why});
    var r0=el.getBoundingClientRect();
    if(r0.top<0||r0.left<0||r0.bottom>innerHeight||r0.right>innerWidth)el.scrollIntoView({block:'center',inline:'center'});
    function bx(){var r=el.getBoundingClientRect();return [r.x,r.y,r.width,r.height];}
    var a=bx();return new Promise(function(res){requestAnimationFrame(function(){var b=bx();var stable=Math.abs(a[0]-b[0])<0.5&&Math.abs(a[1]-b[1])<0.5&&Math.abs(a[2]-b[2])<0.5&&Math.abs(a[3]-b[3])<0.5;if(!stable){res({ok:false,reason:'unstable'});return;}var r=el.getBoundingClientRect();if(r.width<=0&&r.height<=0){res({ok:false,reason:'zero size'});return;}var cx=r.x+r.width/2,cy=r.y+r.height/2;if(cx<0||cy<0||cx>innerWidth||cy>innerHeight){res({ok:false,reason:'offscreen'});return;}var top=document.elementFromPoint(cx,cy);var hit=!top||el===top||el.contains(top)||top.contains(el);if(!hit){var cn=top&&typeof top.className==='string'?top.className.trim().split(/\s+/)[0]:'';res({ok:false,reason:'obscured',coveredBy:top?top.tagName.toLowerCase()+(top.id?'#'+top.id:'')+(cn?'.'+cn:''):''});return;}res({ok:true,box:{x:r.x,y:r.y,width:r.width,height:r.height}});});});
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
    #[serde(rename = "unstable")]
    Unstable,
    #[serde(rename = "zero size")]
    ZeroSize,
    #[serde(rename = "offscreen")]
    Offscreen,
    #[serde(rename = "obscured")]
    Obscured,
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
}

impl Read<'_> {
    pub(crate) fn expr(&self, element: &str) -> String {
        let body = match self {
            Self::BoundingBox => "if(!e)return null;const r=e.getBoundingClientRect();return (r.width<=0&&r.height<=0)?null:{x:r.x,y:r.y,width:r.width,height:r.height};".to_owned(),
            Self::Visible => "if(!e)return false;const r=e.getBoundingClientRect();const s=getComputedStyle(e);return r.width>0&&r.height>0&&s.visibility!=='hidden'&&s.display!=='none';".to_owned(),
            Self::InputValue => "return e?(e.value??null):null;".to_owned(),
            Self::TextContent => "return e?(e.textContent??''):null;".to_owned(),
            Self::InnerText => "return e?(e.innerText??''):null;".to_owned(),
            Self::Attribute(name) => format!(
                "return e?e.getAttribute({}):null;",
                crate::selector::quote(name)
            ),
            Self::Enabled => "if(!e)return false;return !e.disabled&&e.getAttribute('aria-disabled')!=='true'&&!(e.closest&&e.closest('fieldset[disabled]'));".to_owned(),
            Self::Checked => "if(!e)return false;return !!e.checked||e.getAttribute('aria-checked')==='true';".to_owned(),
            Self::Focused => "return !!e&&document.activeElement===e;".to_owned(),
            Self::Attached => "return !!e;".to_owned(),
        };
        format!("(()=>{{const e={element};{body}}})()")
    }
}
