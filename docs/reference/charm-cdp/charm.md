# Module `charm`

| Item | Kind | Description |
| --- | --- | --- |
| [`ActionOptions`](#actionoptions) | struct |  |
| [`Actionable`](#actionable) | struct |  |
| [`BoundingBox`](#boundingbox) | struct |  |
| [`ClickOptions`](#clickoptions) | struct |  |
| [`ClickResult`](#clickresult) | struct |  |
| [`Clip`](#clip) | struct |  |
| [`Conditions`](#conditions) | struct |  |
| [`ConsoleMessage`](#consolemessage) | struct |  |
| [`Cookie`](#cookie) | struct |  |
| [`DeviceSpec`](#devicespec) | struct |  |
| [`Dialog`](#dialog) | struct |  |
| [`Download`](#download) | struct |  |
| [`Events`](#events) | struct |  |
| [`Filter`](#filter) | struct |  |
| [`FrameInfo`](#frameinfo) | struct |  |
| [`FrameLocator`](#framelocator) | struct |  |
| [`Geolocation`](#geolocation) | struct |  |
| [`Launch`](#launch) | struct |  |
| [`Locator`](#locator) | struct |  |
| [`LocatorExpect`](#locatorexpect) | struct |  |
| [`Margins`](#margins) | struct |  |
| [`Modifiers`](#modifiers) | struct |  |
| [`NodeRef`](#noderef) | struct |  |
| [`ObservedRequest`](#observedrequest) | struct |  |
| [`ObservedResponse`](#observedresponse) | struct |  |
| [`OriginStorage`](#originstorage) | struct |  |
| [`Outcome`](#outcome) | struct |  |
| [`PageExpect`](#pageexpect) | struct |  |
| [`PageId`](#pageid) | struct |  |
| [`PageInfo`](#pageinfo) | struct |  |
| [`PdfOptions`](#pdfoptions) | struct |  |
| [`ScreenshotOptions`](#screenshotoptions) | struct |  |
| [`Session`](#session) | struct |  |
| [`Snapshot`](#snapshot) | struct |  |
| [`SnapshotNode`](#snapshotnode) | struct |  |
| [`SnapshotOptions`](#snapshotoptions) | struct |  |
| [`StorageEntry`](#storageentry) | struct |  |
| [`StorageState`](#storagestate) | struct |  |
| [`Timing`](#timing) | struct |  |
| [`Viewport`](#viewport) | struct |  |
| [`ActReport`](#actreport) | enum |  |
| [`Action`](#action) | enum |  |
| [`Blocker`](#blocker) | enum |  |
| [`Checked`](#checked) | enum |  |
| [`ColorScheme`](#colorscheme) | enum |  |
| [`ConsoleLevel`](#consolelevel) | enum |  |
| [`Device`](#device) | enum |  |
| [`DialogKind`](#dialogkind) | enum |  |
| [`DialogPolicy`](#dialogpolicy) | enum |  |
| [`DownloadState`](#downloadstate) | enum |  |
| [`Endpoint`](#endpoint) | enum |  |
| [`Error`](#error) | enum |  |
| [`ErrorReason`](#errorreason) | enum |  |
| [`ImageFormat`](#imageformat) | enum |  |
| [`LoadState`](#loadstate) | enum |  |
| [`MouseButton`](#mousebutton) | enum |  |
| [`PaperSize`](#papersize) | enum |  |
| [`Position`](#position) | enum |  |
| [`RouteAction`](#routeaction) | enum |  |
| [`SameSite`](#samesite) | enum |  |
| [`ScriptError`](#scripterror) | enum |  |
| [`ScriptTag`](#scripttag) | enum |  |
| [`SelectBy`](#selectby) | enum |  |
| [`Selector`](#selector) | enum |  |
| [`TabAction`](#tabaction) | enum |  |
| [`TextFormat`](#textformat) | enum |  |
| [`TextMatch`](#textmatch) | enum |  |
| [`Throttle`](#throttle) | enum |  |
| [`UrlMatch`](#urlmatch) | enum |  |
| [`WaitFor`](#waitfor) | enum |  |
| [`expect`](#expect) | fn |  |
| [`expect_page`](#expect_page) | fn |  |
| [`resolve`](#resolve) | fn |  |
| [`FORBIDDEN_METHODS`](#forbidden_methods) | const |  |

## Structs

### `ActionOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>force: bool</code> |  |
| <code>timeout: Option&lt;Duration&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


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


### `ClickOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>button: <a href="#mousebutton">MouseButton</a></code> |  |
| <code>count: u8</code> |  |
| <code>modifiers: <a href="#modifiers">Modifiers</a></code> |  |
| <code>position: Option&lt;(f64, f64)&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


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


### `Conditions`

**Fields**

| Field | Description |
| --- | --- |
| <code>download_bytes_per_sec: f64</code> |  |
| <code>latency_ms: f64</code> |  |
| <code>upload_bytes_per_sec: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `ConsoleMessage`

**Fields**

| Field | Description |
| --- | --- |
| <code>level: <a href="#consolelevel">ConsoleLevel</a></code> |  |
| <code>line: u64</code> |  |
| <code>text: String</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


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


### `DeviceSpec`

**Fields**

| Field | Description |
| --- | --- |
| <code>name: &amp;'static str</code> |  |
| <code>touch: bool</code> |  |
| <code>user_agent: &amp;'static str</code> |  |
| <code>viewport: <a href="#viewport">Viewport</a></code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Dialog`

**Fields**

| Field | Description |
| --- | --- |
| <code>default_prompt: String</code> |  |
| <code>kind: <a href="#dialogkind">DialogKind</a></code> |  |
| <code>message: String</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Download`

**Fields**

| Field | Description |
| --- | --- |
| <code>guid: String</code> |  |
| <code>path: Option&lt;PathBuf&gt;</code> |  |
| <code>state: <a href="#downloadstate">DownloadState</a></code> |  |
| <code>suggested_filename: String</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Events`

**Methods**

| Method | Description |
| --- | --- |
| <code>all() -&gt; Self</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>page: bool</code> |  |
| <code>runtime: bool</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Filter`

**Fields**

| Field | Description |
| --- | --- |
| <code>has: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>has_not: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>has_not_text: Option&lt;<a href="#textmatch">TextMatch</a>&gt;</code> |  |
| <code>has_text: Option&lt;<a href="#textmatch">TextMatch</a>&gt;</code> |  |
| <code>visible: Option&lt;bool&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `FrameInfo`

**Fields**

| Field | Description |
| --- | --- |
| <code>id: String</code> |  |
| <code>name: Option&lt;String&gt;</code> |  |
| <code>parent_id: Option&lt;String&gt;</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `FrameLocator`

**Methods**

| Method | Description |
| --- | --- |
| <code>pub fn <a href="#framelocator">FrameLocator</a>&lt;'a&gt;::frame_locator(self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Self</code> |  |
| <code>pub fn <a href="#framelocator">FrameLocator</a>&lt;'a&gt;::locator(self, inner: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; <a href="#locator">Locator</a>&lt;'a&gt;</code> |  |


### `Geolocation`

**Fields**

| Field | Description |
| --- | --- |
| <code>accuracy: f64</code> |  |
| <code>latitude: f64</code> |  |
| <code>longitude: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Launch`

**Methods**

| Method | Description |
| --- | --- |
| <code>arg(self, arg: impl Into&lt;String&gt;) -&gt; Self</code> |  |
| <code>args(&amp;self, user_data_dir: &amp;Path) -&gt; Vec&lt;String&gt;</code> |  |
| <code>dialogs(self, policy: <a href="#dialogpolicy">DialogPolicy</a>) -&gt; Self</code> |  |
| <code>events(self, events: <a href="#events">Events</a>) -&gt; Self</code> |  |
| <code>headed(self) -&gt; Self</code> |  |
| <code>installed() -&gt; Result&lt;Self, <a href="#error">Error</a>&gt;</code> |  |
| <code>new(exe: impl Into&lt;PathBuf&gt;) -&gt; Self</code> |  |
| <code>offscreen(self) -&gt; Self</code> |  |
| <code>profile(self, dir: impl Into&lt;PathBuf&gt;) -&gt; Self</code> |  |
| <code>proxy(self, url: impl Into&lt;String&gt;) -&gt; Self</code> |  |
| <code>storage_state(self, state: <a href="#storagestate">StorageState</a>) -&gt; Self</code> |  |
| <code>timing(self, timing: <a href="#timing">Timing</a>) -&gt; Self</code> |  |
| <code>url(self, url: impl Into&lt;String&gt;) -&gt; Self</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Debug</code>


### `Locator`

**Methods**

| Method | Description |
| --- | --- |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::and(self, other: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::filter(self, filter: <a href="#filter">Filter</a>) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::last(self) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::locator(self, inner: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::or(self, other: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'_&gt;::with_options(self, options: <a href="#actionoptions">ActionOptions</a>) -&gt; Self</code> |  |
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
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::blur(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::check(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::clear(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::click_with(&amp;mut self, options: <a href="#clickoptions">ClickOptions</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::dblclick(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::drag_to(&amp;mut self, target: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::drag_to_point(&amp;mut self, x: f64, y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::focus(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::press(&amp;mut self, key: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::select_option(&amp;mut self, by: <a href="#selectby">SelectBy</a>) -&gt; Result&lt;Vec&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::set_checked(&amp;mut self, checked: bool) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::set_input_files(&amp;mut self, files: Vec&lt;PathBuf&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::uncheck(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::click(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::fill(&amp;mut self, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::hover(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::try_click(&amp;mut self) -&gt; Result&lt;<a href="#clickresult">ClickResult</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'_&gt;::type_text(&amp;mut self, text: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'a&gt;::all(&amp;mut self) -&gt; Result&lt;Range&lt;usize&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'a&gt;::count(&amp;mut self) -&gt; Result&lt;usize, <a href="#error">Error</a>&gt;</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'a&gt;::first(self) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'a&gt;::nth(self, index: usize) -&gt; Self</code> |  |
| <code>pub fn <a href="#locator">Locator</a>&lt;'a&gt;::selector(&amp;self) -&gt; &amp;<a href="#selector">Selector</a></code> |  |
| <code>pub async fn <a href="#locator">Locator</a>&lt;'a&gt;::settle(&amp;mut self) -&gt; Result&lt;<a href="#actionable">Actionable</a>, <a href="#error">Error</a>&gt;</code> |  |


### `LocatorExpect`

**Methods**

| Method | Description |
| --- | --- |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_attached(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_checked(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_disabled(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_editable(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_empty(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_enabled(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_focused(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_hidden(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_in_viewport(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_be_visible(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_contain_text(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_attribute(self, name: &amp;str, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_class(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_count(self, count: usize) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_css(self, property: &amp;str, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_id(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_js_property(self, name: &amp;str, expected: &amp;Value) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_text(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_have_value(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#locatorexpect">LocatorExpect</a>&lt;'_, '_&gt;::to_match_aria_snapshot(self, expected: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |

**Trait implementations:** <code>Not</code>


### `Margins`

**Fields**

| Field | Description |
| --- | --- |
| <code>bottom: f64</code> |  |
| <code>left: f64</code> |  |
| <code>right: f64</code> |  |
| <code>top: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Modifiers`

**Methods**

| Method | Description |
| --- | --- |
| <code>const ALT: Self</code> |  |
| <code>const CONTROL: Self</code> |  |
| <code>const META: Self</code> |  |
| <code>const NONE: Self</code> |  |
| <code>const SHIFT: Self</code> |  |
| <code>const bits(self) -&gt; u8</code> |  |
| <code>const contains(self, other: Self) -&gt; bool</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Hash</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>BitOr</code>


### `NodeRef`

**Methods**

| Method | Description |
| --- | --- |
| <code>parse(raw: &amp;str) -&gt; Option&lt;Self&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>Ord</code>, <code>PartialEq</code>, <code>PartialOrd</code>, <code>Debug</code>, <code>Display</code>, <code>Hash</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


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


### `OriginStorage`

**Fields**

| Field | Description |
| --- | --- |
| <code>local_storage: Vec&lt;<a href="#storageentry">StorageEntry</a>&gt;</code> |  |
| <code>origin: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `Outcome`

**Fields**

| Field | Description |
| --- | --- |
| <code>console: Option&lt;Vec&lt;<a href="#consolemessage">ConsoleMessage</a>&gt;&gt;</code> |  |
| <code>dialogs: Option&lt;Vec&lt;<a href="#dialog">Dialog</a>&gt;&gt;</code> |  |
| <code>new_pages: Vec&lt;<a href="#pageid">PageId</a>&gt;</code> |  |
| <code>snapshot: Option&lt;String&gt;</code> |  |
| <code>title: String</code> |  |
| <code>url_after: String</code> |  |
| <code>url_before: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `PageExpect`

**Methods**

| Method | Description |
| --- | --- |
| <code>pub async fn <a href="#pageexpect">PageExpect</a>&lt;'_&gt;::to_have_title(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>pub async fn <a href="#pageexpect">PageExpect</a>&lt;'_&gt;::to_have_url(self, expected: impl Into&lt;<a href="#textmatch">TextMatch</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |

**Trait implementations:** <code>Not</code>


### `PageId`

**Methods**

| Method | Description |
| --- | --- |
| <code>as_str(&amp;self) -&gt; &amp;str</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>Ord</code>, <code>PartialEq</code>, <code>PartialOrd</code>, <code>Debug</code>, <code>Display</code>, <code>Hash</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `PageInfo`

**Fields**

| Field | Description |
| --- | --- |
| <code>active: bool</code> |  |
| <code>id: <a href="#pageid">PageId</a></code> |  |
| <code>title: String</code> |  |
| <code>url: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>


### `PdfOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>landscape: bool</code> |  |
| <code>margins: Option&lt;<a href="#margins">Margins</a>&gt;</code> |  |
| <code>page_ranges: Option&lt;String&gt;</code> |  |
| <code>paper: <a href="#papersize">PaperSize</a></code> |  |
| <code>print_background: bool</code> |  |
| <code>scale: Option&lt;f64&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


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
| <code>async act(&amp;mut self, action: <a href="#action">Action</a>, report: <a href="#actreport">ActReport</a>) -&gt; Result&lt;<a href="#outcome">Outcome</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>active_page(&amp;self) -&gt; &amp;<a href="#pageid">PageId</a></code> |  |
| <code>async close_page(&amp;mut self, id: &amp;<a href="#pageid">PageId</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async new_page(&amp;mut self, url: Option&lt;&amp;str&gt;) -&gt; Result&lt;<a href="#pageid">PageId</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async page_info(&amp;mut self, id: &amp;<a href="#pageid">PageId</a>) -&gt; Result&lt;<a href="#pageinfo">PageInfo</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async pages(&amp;mut self) -&gt; Result&lt;Vec&lt;<a href="#pageinfo">PageInfo</a>&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async select_page(&amp;mut self, id: &amp;<a href="#pageid">PageId</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_page(&amp;mut self) -&gt; Result&lt;<a href="#pageid">PageId</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async add_cookies(&amp;mut self, cookies: Vec&lt;<a href="#cookie">Cookie</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async clear_cookies(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async cookie_named(&amp;mut self, name: &amp;str) -&gt; Result&lt;Option&lt;<a href="#cookie">Cookie</a>&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async cookies(&amp;mut self) -&gt; Result&lt;Vec&lt;<a href="#cookie">Cookie</a>&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async response_body(&amp;mut self, response: &amp;<a href="#observedresponse">ObservedResponse</a>) -&gt; Result&lt;Vec&lt;u8&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_request(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>) -&gt; Result&lt;<a href="#observedrequest">ObservedRequest</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_response(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>) -&gt; Result&lt;<a href="#observedresponse">ObservedResponse</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async add_script_tag(&amp;mut self, tag: <a href="#scripttag">ScriptTag</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async content(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async goto(&amp;mut self, url: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async open(&amp;mut self, url: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async reload(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async screenshot(&amp;mut self, options: <a href="#screenshotoptions">ScreenshotOptions</a>) -&gt; Result&lt;Vec&lt;u8&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async title(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async url(&amp;mut self) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async block_urls(&amp;mut self, urls: &amp;[String]) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async browser_send(&amp;mut self, method: &amp;str, params: Value) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>async close(self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async connect(endpoint: <a href="#endpoint">Endpoint</a>, timing: <a href="#timing">Timing</a>) -&gt; Result&lt;Self, <a href="#error">Error</a>&gt;</code> |  |
| <code>dropped_events(&amp;self) -&gt; u64</code> |  |
| <code>async evaluate(&amp;mut self, expression: &amp;str) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>exited(&amp;mut self) -&gt; bool</code> |  |
| <code>async keep_cookies(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async launch(launch: <a href="#launch">Launch</a>) -&gt; Result&lt;Self, <a href="#error">Error</a>&gt;</code> |  |
| <code>async network_on(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>reply_failures(&amp;self) -&gt; u64</code> |  |
| <code>async send(&amp;mut self, method: &amp;str, params: Value) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>sent(&amp;self) -&gt; &amp;BTreeSet&lt;String&gt;</code> |  |
| <code>async take_events(&amp;mut self) -&gt; Result&lt;Vec&lt;Value&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async check(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async dblclick(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async drag_and_drop(&amp;mut self, source: impl Into&lt;<a href="#selector">Selector</a>&gt;, target: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async focus(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async select_option(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;, by: <a href="#selectby">SelectBy</a>) -&gt; Result&lt;Vec&lt;String&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_input_files(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;, files: Vec&lt;PathBuf&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async uncheck(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async clear_viewport(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async emulate(&amp;mut self, device: <a href="#device">Device</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async pdf(&amp;mut self, options: <a href="#pdfoptions">PdfOptions</a>) -&gt; Result&lt;Vec&lt;u8&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_color_scheme(&amp;mut self, scheme: <a href="#colorscheme">ColorScheme</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_extra_headers(&amp;mut self, headers: BTreeMap&lt;String, String&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_geolocation(&amp;mut self, location: Option&lt;<a href="#geolocation">Geolocation</a>&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_javascript_enabled(&amp;mut self, enabled: bool) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_locale(&amp;mut self, locale: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_offline(&amp;mut self, offline: bool) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_throttle(&amp;mut self, throttle: <a href="#throttle">Throttle</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_timezone(&amp;mut self, timezone: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_user_agent(&amp;mut self, user_agent: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_viewport(&amp;mut self, viewport: <a href="#viewport">Viewport</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
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
| <code>async click_at(&amp;mut self, x: f64, y: f64, options: <a href="#clickoptions">ClickOptions</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async mouse_click(&amp;mut self, x: f64, y: f64, button: <a href="#mousebutton">MouseButton</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async mouse_move(&amp;mut self, x: f64, y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async press(&amp;mut self, combo: &amp;str) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async scroll(&amp;mut self, delta_y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wheel(&amp;mut self, delta_x: f64, delta_y: f64) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>console(&amp;mut self) -&gt; Vec&lt;<a href="#consolemessage">ConsoleMessage</a>&gt;</code> |  |
| <code>dialogs(&amp;mut self) -&gt; Vec&lt;<a href="#dialog">Dialog</a>&gt;</code> |  |
| <code>dropped_records(&amp;mut self) -&gt; u64</code> |  |
| <code>async set_events(&amp;mut self, events: <a href="#events">Events</a>, policy: <a href="#dialogpolicy">DialogPolicy</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>downloads(&amp;mut self) -&gt; Vec&lt;<a href="#download">Download</a>&gt;</code> |  |
| <code>async on_file_chooser(&amp;mut self, files: Vec&lt;PathBuf&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async set_downloads(&amp;mut self, dir: PathBuf) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_download(&amp;mut self) -&gt; Result&lt;<a href="#download">Download</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>frame_locator(&amp;mut self, selector: impl Into&lt;<a href="#selector">Selector</a>&gt;) -&gt; <a href="#framelocator">FrameLocator</a>&lt;'_&gt;</code> |  |
| <code>async frames(&amp;mut self) -&gt; Result&lt;Vec&lt;<a href="#frameinfo">FrameInfo</a>&gt;, <a href="#error">Error</a>&gt;</code> |  |
| <code>async go_back(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async go_forward(&amp;mut self) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async route(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>, action: <a href="#routeaction">RouteAction</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async unroute(&amp;mut self, url: <a href="#urlmatch">UrlMatch</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>set_human_input(&amp;mut self, cursor: Option&lt;<a href="charm-human.html#cursoroptions">CursorOptions</a>&gt;)</code> |  |
| <code>async snapshot(&amp;mut self, options: <a href="#snapshotoptions">SnapshotOptions</a>) -&gt; Result&lt;<a href="#snapshot">Snapshot</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async storage_state(&amp;mut self) -&gt; Result&lt;<a href="#storagestate">StorageState</a>, <a href="#error">Error</a>&gt;</code> |  |
| <code>async text(&amp;mut self, format: <a href="#textformat">TextFormat</a>, max_chars: usize) -&gt; Result&lt;String, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_function(&amp;mut self, expression: &amp;str) -&gt; Result&lt;Value, <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_load_state(&amp;mut self, state: <a href="#loadstate">LoadState</a>) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |
| <code>async wait_for_timeout(&amp;mut self, duration: Duration)</code> |  |

**Trait implementations:** <code>Drop</code>


### `Snapshot`

**Methods**

| Method | Description |
| --- | --- |
| <code>diff(&amp;self, previous: &amp;<a href="#snapshot">Snapshot</a>) -&gt; String</code> |  |
| <code>header(&amp;self) -&gt; String</code> |  |
| <code>nodes(&amp;self) -&gt; &amp;[<a href="#snapshotnode">SnapshotNode</a>]</code> |  |
| <code>page(&amp;self) -&gt; &amp;<a href="#pageid">PageId</a></code> |  |
| <code>text(&amp;self) -&gt; String</code> |  |
| <code>title(&amp;self) -&gt; &amp;str</code> |  |
| <code>truncated(&amp;self) -&gt; bool</code> |  |
| <code>url(&amp;self) -&gt; &amp;str</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `SnapshotNode`

**Fields**

| Field | Description |
| --- | --- |
| <code>checked: Option&lt;<a href="#checked">Checked</a>&gt;</code> |  |
| <code>depth: usize</code> |  |
| <code>disabled: bool</code> |  |
| <code>expanded: Option&lt;bool&gt;</code> |  |
| <code>level: Option&lt;u64&gt;</code> |  |
| <code>name: String</code> |  |
| <code>node_ref: Option&lt;<a href="#noderef">NodeRef</a>&gt;</code> |  |
| <code>role: String</code> |  |
| <code>selected: bool</code> |  |
| <code>url: Option&lt;String&gt;</code> |  |
| <code>value: Option&lt;String&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Display</code>, <code>StructuralPartialEq</code>


### `SnapshotOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>depth: Option&lt;usize&gt;</code> |  |
| <code>frames: bool</code> |  |
| <code>interactive_only: bool</code> |  |
| <code>interesting_only: bool</code> |  |
| <code>max_chars: Option&lt;usize&gt;</code> |  |
| <code>max_nodes: usize</code> |  |
| <code>scope: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `StorageEntry`

**Fields**

| Field | Description |
| --- | --- |
| <code>name: String</code> |  |
| <code>value: String</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `StorageState`

**Methods**

| Method | Description |
| --- | --- |
| <code>load(path: impl AsRef&lt;Path&gt;) -&gt; Result&lt;Self, <a href="#error">Error</a>&gt;</code> |  |
| <code>save(&amp;self, path: impl AsRef&lt;Path&gt;) -&gt; Result&lt;(), <a href="#error">Error</a>&gt;</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>cookies: Vec&lt;<a href="#cookie">Cookie</a>&gt;</code> |  |
| <code>origins: Vec&lt;<a href="#originstorage">OriginStorage</a>&gt;</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


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
| <code>network_idle: Duration</code> |  |
| <code>poll: Duration</code> |  |
| <code>settle: Duration</code> |  |
| <code>wait: Duration</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Viewport`

**Fields**

| Field | Description |
| --- | --- |
| <code>device_scale_factor: f64</code> |  |
| <code>height: u32</code> |  |
| <code>mobile: bool</code> |  |
| <code>width: u32</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


## Enums

### `ActReport`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Delta</code> |  |
| <code>Full</code> |  |
| <code>None</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `Action`

**Fields**

| Field | Description |
| --- | --- |
| <code>Check::checked: bool</code> |  |
| <code>Check::target: <a href="#selector">Selector</a></code> |  |
| <code>Click::options: <a href="#clickoptions">ClickOptions</a></code> |  |
| <code>Click::target: <a href="#selector">Selector</a></code> |  |
| <code>Drag::from: <a href="#selector">Selector</a></code> |  |
| <code>Drag::to: <a href="#selector">Selector</a></code> |  |
| <code>Fill::target: <a href="#selector">Selector</a></code> |  |
| <code>Fill::text: String</code> |  |
| <code>Hover::target: <a href="#selector">Selector</a></code> |  |
| <code>Navigate::url: String</code> |  |
| <code>Press::key: String</code> |  |
| <code>Press::target: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Scroll::delta_y: f64</code> |  |
| <code>Scroll::target: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Select::by: <a href="#selectby">SelectBy</a></code> |  |
| <code>Select::target: <a href="#selector">Selector</a></code> |  |
| <code>Type::target: <a href="#selector">Selector</a></code> |  |
| <code>Type::text: String</code> |  |
| <code>Upload::files: Vec&lt;PathBuf&gt;</code> |  |
| <code>Upload::target: <a href="#selector">Selector</a></code> |  |
| <code>Wait::for_: <a href="#waitfor">WaitFor</a></code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Back</code> |  |
| <code>Check</code> |  |
| <code>Click</code> |  |
| <code>Drag</code> |  |
| <code>Fill</code> |  |
| <code>Forward</code> |  |
| <code>Hover</code> |  |
| <code>Navigate</code> |  |
| <code>Press</code> |  |
| <code>Reload</code> |  |
| <code>Scroll</code> |  |
| <code>Select</code> |  |
| <code>Tab(<a href="#tabaction">TabAction</a>)</code> |  |
| <code>Type</code> |  |
| <code>Upload</code> |  |
| <code>Wait</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `Blocker`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Detached</code> |  |
| <code>Disabled</code> |  |
| <code>NoMatch</code> |  |
| <code>NotEditable</code> |  |
| <code>NotVisible</code> |  |
| <code>Obscured</code> |  |
| <code>Offscreen</code> |  |
| <code>Unstable</code> |  |
| <code>ZeroSize</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `Checked`

**Variants**

| Variant | Description |
| --- | --- |
| <code>False</code> |  |
| <code>Mixed</code> |  |
| <code>True</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `ColorScheme`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Dark</code> |  |
| <code>Light</code> |  |
| <code>NoPreference</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `ConsoleLevel`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Debug</code> |  |
| <code>Error</code> |  |
| <code>Exception</code> |  |
| <code>Info</code> |  |
| <code>Log</code> |  |
| <code>Warning</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Device`

**Methods**

| Method | Description |
| --- | --- |
| <code>named(name: &amp;str) -&gt; Option&lt;Self&gt;</code> |  |
| <code>spec(self) -&gt; <a href="#devicespec">DeviceSpec</a></code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Desktop1080p</code> |  |
| <code>Ipad</code> |  |
| <code>Iphone15</code> |  |
| <code>Pixel8</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Hash</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `DialogKind`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Alert</code> |  |
| <code>BeforeUnload</code> |  |
| <code>Confirm</code> |  |
| <code>Prompt</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `DialogPolicy`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Accept</code> |  |
| <code>AcceptWith(String)</code> |  |
| <code>Dismiss</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `DownloadState`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Canceled</code> |  |
| <code>Completed</code> |  |
| <code>InProgress</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Endpoint`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Port(u16)</code> |  |
| <code>WebSocket(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Error`

**Fields**

| Field | Description |
| --- | --- |
| <code>Assertion::expected: String</code> |  |
| <code>Assertion::received: String</code> |  |
| <code>Assertion::selector: Option&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Exited::code: Option&lt;i32&gt;</code> |  |
| <code>Exited::stderr: String</code> |  |
| <code>NotActionable::covered_by: Option&lt;String&gt;</code> |  |
| <code>NotActionable::reason: Option&lt;<a href="#blocker">Blocker</a>&gt;</code> |  |
| <code>NotActionable::selector: <a href="#selector">Selector</a></code> |  |
| <code>ProfileInUse::path: PathBuf</code> |  |
| <code>Protocol::code: Option&lt;i64&gt;</code> |  |
| <code>Protocol::message: String</code> |  |
| <code>Protocol::method: String</code> |  |
| <code>Script::kind: <a href="#scripterror">ScriptError</a></code> |  |
| <code>Script::message: String</code> |  |
| <code>Spawn::path: PathBuf</code> |  |
| <code>Spawn::source: Error</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Assertion</code> |  |
| <code>Blocked(&amp;'static str)</code> |  |
| <code>Closed</code> |  |
| <code>Connect(String)</code> |  |
| <code>EventsRequired(&amp;'static str)</code> |  |
| <code>Exited</code> |  |
| <code>Frame(String)</code> |  |
| <code>Io(Error)</code> |  |
| <code>Json(Error)</code> |  |
| <code>MissingExe</code> |  |
| <code>NotActionable</code> |  |
| <code>ProfileInUse</code> |  |
| <code>Protocol</code> |  |
| <code>Script</code> |  |
| <code>Spawn</code> |  |
| <code>StaleRef(String)</code> |  |
| <code>Timeout(String)</code> |  |
| <code>Ws(Box&lt;Error&gt;)</code> |  |

**Trait implementations:** <code>From&lt;Error&gt;</code>, <code>Error</code>, <code>Debug</code>, <code>Display</code>


### `ErrorReason`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Aborted</code> |  |
| <code>AccessDenied</code> |  |
| <code>AddressUnreachable</code> |  |
| <code>BlockedByClient</code> |  |
| <code>BlockedByResponse</code> |  |
| <code>ConnectionAborted</code> |  |
| <code>ConnectionClosed</code> |  |
| <code>ConnectionFailed</code> |  |
| <code>ConnectionRefused</code> |  |
| <code>ConnectionReset</code> |  |
| <code>Failed</code> |  |
| <code>InternetDisconnected</code> |  |
| <code>NameNotResolved</code> |  |
| <code>TimedOut</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>


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
| <code>NetworkIdle</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `MouseButton`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Left</code> |  |
| <code>Middle</code> |  |
| <code>Right</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `PaperSize`

**Methods**

| Method | Description |
| --- | --- |
| <code>inches(self) -&gt; (f64, f64)</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>A3</code> |  |
| <code>A4</code> |  |
| <code>A5</code> |  |
| <code>Legal</code> |  |
| <code>Letter</code> |  |
| <code>Tabloid</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Position`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Index(usize)</code> |  |
| <code>Last</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `RouteAction`

**Fields**

| Field | Description |
| --- | --- |
| <code>Continue::headers: Option&lt;BTreeMap&lt;String, String&gt;&gt;</code> |  |
| <code>Continue::method: Option&lt;String&gt;</code> |  |
| <code>Continue::post_data: Option&lt;Vec&lt;u8&gt;&gt;</code> |  |
| <code>Continue::url: Option&lt;String&gt;</code> |  |
| <code>Fulfill::body: Vec&lt;u8&gt;</code> |  |
| <code>Fulfill::headers: BTreeMap&lt;String, String&gt;</code> |  |
| <code>Fulfill::status: u16</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Abort(<a href="#errorreason">ErrorReason</a>)</code> |  |
| <code>Continue</code> |  |
| <code>Fulfill</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `SameSite`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Lax</code> |  |
| <code>None</code> |  |
| <code>Strict</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>, <code>Serialize</code>, <code>Deserialize&lt;'de&gt;</code>


### `ScriptError`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Other</code> |  |
| <code>Syntax</code> |  |
| <code>Type</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `ScriptTag`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Content(String)</code> |  |
| <code>Url(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `SelectBy`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Index(Vec&lt;usize&gt;)</code> |  |
| <code>Label(Vec&lt;String&gt;)</code> |  |
| <code>Value(Vec&lt;String&gt;)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Selector`

**Methods**

| Method | Description |
| --- | --- |
| <code>parse(raw: &amp;str) -&gt; Self</code> |  |
| <code>parse_target(raw: &amp;str) -&gt; Self</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>Filtered::base: Box&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Filtered::filter: Box&lt;<a href="#filter">Filter</a>&gt;</code> |  |
| <code>Nth::at: <a href="#position">Position</a></code> |  |
| <code>Nth::base: Box&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Role::name: Option&lt;<a href="#textmatch">TextMatch</a>&gt;</code> |  |
| <code>Role::role: String</code> |  |
| <code>Within::inner: Box&lt;<a href="#selector">Selector</a>&gt;</code> |  |
| <code>Within::outer: Box&lt;<a href="#selector">Selector</a>&gt;</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>And(Box&lt;<a href="#selector">Selector</a>&gt;, Box&lt;<a href="#selector">Selector</a>&gt;)</code> |  |
| <code>Css(String)</code> |  |
| <code>Filtered</code> |  |
| <code>Label(<a href="#textmatch">TextMatch</a>)</code> |  |
| <code>Nth</code> |  |
| <code>Or(Box&lt;<a href="#selector">Selector</a>&gt;, Box&lt;<a href="#selector">Selector</a>&gt;)</code> |  |
| <code>Ref(String)</code> |  |
| <code>Role</code> |  |
| <code>TestId(String)</code> |  |
| <code>Text(<a href="#textmatch">TextMatch</a>)</code> |  |
| <code>Within</code> |  |
| <code>XPath(String)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>From&lt;&amp;String&gt;</code>, <code>From&lt;&amp;str&gt;</code>, <code>From&lt;String&gt;</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `TabAction`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Close(<a href="#pageid">PageId</a>)</code> |  |
| <code>New(Option&lt;String&gt;)</code> |  |
| <code>Select(<a href="#pageid">PageId</a>)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


### `TextFormat`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Markdown</code> |  |
| <code>Text</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `TextMatch`

**Methods**

| Method | Description |
| --- | --- |
| <code>new(text: &amp;str, exact: bool) -&gt; Self</code> |  |
| <code>regex(pattern: &amp;str, flags: &amp;str) -&gt; Self</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>Regex::flags: String</code> |  |
| <code>Regex::pattern: String</code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Contains(String)</code> |  |
| <code>Exact(String)</code> |  |
| <code>Regex</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>From&lt;&amp;str&gt;</code>, <code>From&lt;String&gt;</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `Throttle`

**Methods**

| Method | Description |
| --- | --- |
| <code>conditions(self) -&gt; <a href="#conditions">Conditions</a></code> |  |

**Variants**

| Variant | Description |
| --- | --- |
| <code>Custom(<a href="#conditions">Conditions</a>)</code> |  |
| <code>Fast3g</code> |  |
| <code>None</code> |  |
| <code>Slow3g</code> |  |
| <code>Slow4g</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


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


### `WaitFor`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Duration(Duration)</code> |  |
| <code>LoadState(<a href="#loadstate">LoadState</a>)</code> |  |
| <code>Selector(<a href="#selector">Selector</a>)</code> |  |
| <code>Text(<a href="#textmatch">TextMatch</a>)</code> |  |
| <code>Url(<a href="#urlmatch">UrlMatch</a>)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>StructuralPartialEq</code>, <code>Deserialize&lt;'de&gt;</code>


## Functions

### `expect`

<pre>pub fn <a href="#expect">expect</a>&lt;'l, 's&gt;(locator: &amp;'l mut <a href="#locator">Locator</a>&lt;'s&gt;) -&gt; <a href="#locatorexpect">LocatorExpect</a>&lt;'l, 's&gt;</pre>


### `expect_page`

<pre>pub fn <a href="#expect_page">expect_page</a>(session: &amp;mut <a href="#session">Session</a>) -&gt; <a href="#pageexpect">PageExpect</a>&lt;'_&gt;</pre>


### `resolve`

<pre>pub fn <a href="#resolve">resolve</a>() -&gt; Option&lt;PathBuf&gt;</pre>


## Constants

### `FORBIDDEN_METHODS`

<pre>pub const <a href="#forbidden_methods">FORBIDDEN_METHODS</a>: &amp;[&amp;str]</pre>


---

Generated by truesight from `charm-cdp` 0.1.0. See the [API overview](index.md).
