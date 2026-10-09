# Module `charm`

| Item | Kind | Description |
| --- | --- | --- |
| [`Actionable`](#actionable) | struct |  |
| [`BoundingBox`](#boundingbox) | struct |  |
| [`ClickResult`](#clickresult) | struct |  |
| [`Clip`](#clip) | struct |  |
| [`Cookie`](#cookie) | struct |  |
| [`Launch`](#launch) | struct |  |
| [`Locator`](#locator) | struct |  |
| [`ObservedRequest`](#observedrequest) | struct |  |
| [`ObservedResponse`](#observedresponse) | struct |  |
| [`ScreenshotOptions`](#screenshotoptions) | struct |  |
| [`Session`](#session) | struct |  |
| [`Timing`](#timing) | struct |  |
| [`Blocker`](#blocker) | enum |  |
| [`Error`](#error) | enum |  |
| [`ImageFormat`](#imageformat) | enum |  |
| [`LoadState`](#loadstate) | enum |  |
| [`MouseButton`](#mousebutton) | enum |  |
| [`SameSite`](#samesite) | enum |  |
| [`ScriptTag`](#scripttag) | enum |  |
| [`Selector`](#selector) | enum |  |
| [`UrlMatch`](#urlmatch) | enum |  |
| [`resolve`](#resolve) | fn |  |
| [`FORBIDDEN_METHODS`](#forbidden_methods) | const |  |

## Structs

### `Actionable`

**Fields**

| Field | Description |
| --- | --- |
| <code>bounds: Option&lt;<a href="#boundingbox">BoundingBox</a>&gt;</code> |  |
| <code>covered_by: Option&lt;String&gt;</code> |  |
| <code>ok: bool</code> |  |
| <code>reason: Option&lt;<a href="#blocker">Blocker</a>&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `BoundingBox`

**Methods**

| Method | Description |
| --- | --- |
| <code>center(&amp;self) -&gt; (f64, f64)</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>height: f64</code> |  |
| <code>width: f64</code> |  |
| <code>x: f64</code> |  |
| <code>y: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `ClickResult`

**Fields**

| Field | Description |
| --- | --- |
| <code>clicked: bool</code> |  |
| <code>covered_by: Option&lt;String&gt;</code> |  |
| <code>reason: Option&lt;<a href="#blocker">Blocker</a>&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Clip`

**Fields**

| Field | Description |
| --- | --- |
| <code>height: f64</code> |  |
| <code>scale: f64</code> |  |
| <code>width: f64</code> |  |
| <code>x: f64</code> |  |
| <code>y: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Cookie`

**Fields**

| Field | Description |
| --- | --- |
| <code>domain: Option&lt;String&gt;</code> |  |
| <code>expires: Option&lt;f64&gt;</code> |  |
| <code>http_only: bool</code> |  |
| <code>name: String</code> |  |
| <code>path: Option&lt;String&gt;</code> |  |
| <code>same_site: Option&lt;<a href="#samesite">SameSite</a>&gt;</code> |  |
| <code>secure: bool</code> |  |
| <code>url: Option&lt;String&gt;</code> |  |
| <code>value: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `Launch`

**Methods**

| Method | Description |
| --- | --- |
| <code>arg(self, arg: impl Into&lt;String&gt;) -&gt; Self</code> |  |
| <code>args(&amp;self, user_data_dir: &amp;Path) -&gt; Vec&lt;String&gt;</code> |  |
| <code>headed(self) -&gt; Self</code> |  |
| <code>new(exe: impl Into&lt;PathBuf&gt;) -&gt; Self</code> |  |
| <code>offscreen(self) -&gt; Self</code> |  |
| <code>profile(self, dir: impl Into&lt;PathBuf&gt;) -&gt; Self</code> |  |
| <code>proxy(self, url: impl Into&lt;String&gt;) -&gt; Self</code> |  |
| <code>timing(self, timing: <a href="#timing">Timing</a>) -&gt; Self</code> |  |
| <code>url(self, url: impl Into&lt;String&gt;) -&gt; Self</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Debug</code>


### `Locator`

**Methods**

| Method | Description |
| --- | --- |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::all(&amp;mut self) -&gt; Result&lt;Range&lt;usize&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::click(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::count(&amp;mut self) -&gt; Result&lt;usize, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::fill(&amp;mut self, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::first(self) -&gt; Self</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::hover(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::nth(self, index: usize) -&gt; Self</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::press_sequentially(&amp;mut self, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::selector(&amp;self) -&gt; &amp;<a href="#selector">Selector</a></code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::settle(&amp;mut self) -&gt; Result&lt;<a href="#actionable">Actionable</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::try_click(&amp;mut self) -&gt; Result&lt;<a href="#clickresult">ClickResult</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::type_text(&amp;mut self, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::attribute(&amp;mut self, name: &amp;str) -&gt; Result&lt;Option&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::bounding_box(&amp;mut self) -&gt; Result&lt;Option&lt;<a href="#boundingbox">BoundingBox</a>&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::inner_text(&amp;mut self) -&gt; Result&lt;Option&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::input_value(&amp;mut self) -&gt; Result&lt;Option&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::is_attached(&amp;mut self) -&gt; Result&lt;bool, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::is_checked(&amp;mut self) -&gt; Result&lt;bool, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::is_enabled(&amp;mut self) -&gt; Result&lt;bool, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::is_focused(&amp;mut self) -&gt; Result&lt;bool, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::is_visible(&amp;mut self) -&gt; Result&lt;bool, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::text_content(&amp;mut self) -&gt; Result&lt;Option&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |


### `ObservedRequest`

**Fields**

| Field | Description |
| --- | --- |
| <code>headers: BTreeMap&lt;String, String&gt;</code> |  |
| <code>method: String</code> |  |
| <code>request_id: String</code> |  |
| <code>resource_type: String</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `ObservedResponse`

**Fields**

| Field | Description |
| --- | --- |
| <code>headers: BTreeMap&lt;String, String&gt;</code> |  |
| <code>request_id: String</code> |  |
| <code>resource_type: String</code> |  |
| <code>status: u16</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `ScreenshotOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>clip: Option&lt;<a href="#clip">Clip</a>&gt;</code> |  |
| <code>format: <a href="#imageformat">ImageFormat</a></code> |  |
| <code>full_page: bool</code> |  |
| <code>quality: Option&lt;u8&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Session`

**Methods**

| Method | Description |
| --- | --- |
| <code>async add_cookies(&amp;mut self, cookies: Vec&lt;<a href="#cookie">Cookie</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async clear_cookies(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_request(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>) -&gt; Result&lt;<a href="#observedrequest">ObservedRequest</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_response(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>) -&gt; Result&lt;<a href="#observedresponse">ObservedResponse</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async add_script_tag(&amp;mut self, tag: <a href="#scripttag">ScriptTag</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async content(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async reload(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async screenshot(&amp;mut self, options: <a href="#screenshotoptions">ScreenshotOptions</a>) -&gt; Result&lt;Vec&lt;u8&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async title(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async url(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async block_urls(&amp;mut self, urls: &amp;[String]) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async browser_send(&amp;mut self, method: &amp;str, params: Value) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>async close(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async cookie_named(&amp;mut self, name: &amp;str) -&gt; Result&lt;Option&lt;Value&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async cookies(&amp;mut self) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>async evaluate(&amp;mut self, expression: &amp;str) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>exited(&amp;mut self) -&gt; bool</code> |  |
| <code>async goto(&amp;mut self, url: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async launch(launch: <a href="#launch">Launch</a>) -&gt; Result&lt;Self, <a href="#error">Error</a>&gt;</code> |  |
| <code>async network_on(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async open(&amp;mut self, url: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async send(&amp;mut self, method: &amp;str, params: Value) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>sent(&amp;self) -&gt; &amp;BTreeSet&lt;String&gt;</code> |  |
| <code>async set_cookies(&amp;mut self, cookies: Value) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async take_events(&amp;mut self) -&gt; Result&lt;Vec&lt;Value&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async click(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async fill(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>get_by_label(&amp;mut self, text: &amp;str, exact: bool) -&gt; <a href="#locator">Locator</a>&lt;'_&gt;</code> |  |
| <code>get_by_role(&amp;mut self, role: &amp;str, name: Option&lt;&amp;str&gt;, exact: bool) -&gt; <a href="#locator">Locator</a>&lt;'_&gt;</code> |  |
| <code>get_by_test_id(&amp;mut self, id: &amp;str) -&gt; <a href="#locator">Locator</a>&lt;'_&gt;</code> |  |
| <code>get_by_text(&amp;mut self, text: &amp;str, exact: bool) -&gt; <a href="#locator">Locator</a>&lt;'_&gt;</code> |  |
| <code>async hover(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>locator(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; <a href="#locator">Locator</a>&lt;'_&gt;</code> |  |
| <code>async type_text(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_selector(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async mouse_click(&amp;mut self, x: f64, y: f64, button: <a href="#mousebutton">MouseButton</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async mouse_move(&amp;mut self, x: f64, y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async press(&amp;mut self, combo: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async scroll(&amp;mut self, delta_y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wheel(&amp;mut self, delta_x: f64, delta_y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_function(&amp;mut self, expression: &amp;str) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_load_state(&amp;mut self, state: <a href="#loadstate">LoadState</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_timeout(&amp;mut self, duration: Duration)</code> |  |

**Trait implementations:** <code>Drop</code>


### `Timing`

**Fields**

| Field | Description |
| --- | --- |
| <code>actionable: Duration</code> |  |
| <code>close: Duration</code> |  |
| <code>command: Duration</code> |  |
| <code>connect: Duration</code> |  |
| <code>connect_poll: Duration</code> |  |
| <code>drain: Duration</code> |  |
| <code>navigate: Duration</code> |  |
| <code>poll: Duration</code> |  |
| <code>settle: Duration</code> |  |
| <code>wait: Duration</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


## Enums

### `Blocker`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Detached</code> |  |
| <code>Disabled</code> |  |
| <code>NoMatch</code> |  |
| <code>NotVisible</code> |  |
| <code>Obscured</code> |  |
| <code>Offscreen</code> |  |
| <code>Unstable</code> |  |
| <code>ZeroSize</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `Error`

**Fields**

| Field | Description |
| --- | --- |
| <code>Protocol::message: String</code> |  |
| <code>Protocol::method: String</code> |  |
| <code>Spawn::path: PathBuf</code> |  |
| <code>Spawn::source: Error</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Blocked(&amp;'static str)</code> |  |
| <code>Connect(String)</code> |  |
| <code>Io(Error)</code> |  |
| <code>Json(Error)</code> |  |
| <code>MissingExe</code> |  |
| <code>Protocol</code> |  |
| <code>Spawn</code> |  |
| <code>Timeout(String)</code> |  |
| <code>Ws(Box&lt;Error&gt;)</code> |  |

**Trait implementations:** <code>From&lt;Error&gt;</code>, <code>Error</code>, <code>Debug</code>, <code>Display</code>


### `ImageFormat`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Jpeg</code> |  |
| <code>Png</code> |  |
| <code>Webp</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `LoadState`

**Variants**

| Variant | Description |
| --- | --- |
| <code>DomContentLoaded</code> |  |
| <code>Load</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `MouseButton`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Left</code> |  |
| <code>Middle</code> |  |
| <code>Right</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `SameSite`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Lax</code> |  |
| <code>None</code> |  |
| <code>Strict</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `ScriptTag`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Content(String)</code> |  |
| <code>Url(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Selector`

**Methods**

| Method | Description |
| --- | --- |
| <code>parse(raw: &amp;str) -&gt; Self</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>Label::exact: bool</code> |  |
| <code>Label::text: String</code> |  |
| <code>Role::exact: bool</code> |  |
| <code>Role::name: Option&lt;String&gt;</code> |  |
| <code>Role::role: String</code> |  |
| <code>Text::exact: bool</code> |  |
| <code>Text::text: String</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Css(String)</code> |  |
| <code>Label</code> |  |
| <code>Role</code> |  |
| <code>TestId(String)</code> |  |
| <code>Text</code> |  |
| <code>XPath(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>From&lt;&amp;String&gt;</code>, <code>From&lt;&amp;str&gt;</code>, <code>From&lt;String&gt;</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `UrlMatch`

**Methods**

| Method | Description |
| --- | --- |
| <code>matches(&amp;self, url: &amp;str) -&gt; bool</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Contains(String)</code> |  |
| <code>Exact(String)</code> |  |
| <code>Glob(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


## Functions

### `resolve`

<pre>pub fn <a href="#resolve">resolve</a>() -&gt; Option&lt;PathBuf&gt;</pre>


## Constants

### `FORBIDDEN_METHODS`

<pre>pub const <a href="#forbidden_methods">FORBIDDEN_METHODS</a>: [&amp;str; 3]</pre>


---

Generated by truesight from `charm-cdp` 0.1.0. See the [API overview](index.md).
