# Module `charm::human`

| Item | Kind | Description |
| --- | --- | --- |
| [`Band`](#band) | struct |  |
| [`CursorOptions`](#cursoroptions) | struct |  |
| [`DelayBand`](#delayband) | struct |  |
| [`Keystroke`](#keystroke) | struct |  |
| [`Point`](#point) | struct |  |
| [`Rect`](#rect) | struct |  |
| [`Rng`](#rng) | struct |  |
| [`Step`](#step) | struct |  |
| [`TypingOptions`](#typingoptions) | struct |  |
| [`Key`](#key) | enum |  |
| [`Target`](#target) | enum |  |
| [`keystrokes`](#keystrokes) | fn |  |
| [`path`](#path) | fn |  |
| [`target_point`](#target_point) | fn |  |

## Structs

### `Band`

**Fields**

| Field | Description |
| --- | --- |
| <code>hi: f64</code> |  |
| <code>lo: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `CursorOptions`

**Fields**

| Field | Description |
| --- | --- |
| <code>arc_skew: <a href="#band">Band</a></code> |  |
| <code>base_submovements: usize</code> |  |
| <code>bow: <a href="#band">Band</a></code> |  |
| <code>bow_flip_rate: f64</code> |  |
| <code>corrective_duration: <a href="#delayband">DelayBand</a></code> |  |
| <code>corrective_share: <a href="#band">Band</a></code> |  |
| <code>default_target_width: f64</code> |  |
| <code>extra_submovement_rate: f64</code> |  |
| <code>fitts_a_ms: f64</code> |  |
| <code>fitts_b_ms: f64</code> |  |
| <code>hesitate: <a href="#delayband">DelayBand</a></code> |  |
| <code>hurry_factor: <a href="#band">Band</a></code> |  |
| <code>hurry_rate: f64</code> |  |
| <code>long_move_distance: f64</code> |  |
| <code>max_padding_percentage: f64</code> |  |
| <code>max_submovements: usize</code> |  |
| <code>min_distance: f64</code> |  |
| <code>min_emit_px: f64</code> |  |
| <code>min_fitts_distance: f64</code> |  |
| <code>min_primary_duration: Duration</code> |  |
| <code>min_target_width: f64</code> |  |
| <code>move_delay: <a href="#delayband">DelayBand</a></code> |  |
| <code>normal_factor: <a href="#band">Band</a></code> |  |
| <code>overlap: <a href="#band">Band</a></code> |  |
| <code>padding_percentage: f64</code> |  |
| <code>primary_duration_share: <a href="#band">Band</a></code> |  |
| <code>primary_share: <a href="#band">Band</a></code> |  |
| <code>sample_interval: <a href="#delayband">DelayBand</a></code> |  |
| <code>settle_delay: <a href="#delayband">DelayBand</a></code> |  |
| <code>settle_nudges_max: u64</code> |  |
| <code>settle_nudges_min: u64</code> |  |
| <code>settle_rate: f64</code> |  |
| <code>settle_return_delay: <a href="#delayband">DelayBand</a></code> |  |
| <code>settle_sd_px: f64</code> |  |
| <code>share_skew_sd: f64</code> |  |
| <code>spread: f64</code> |  |
| <code>stall_at: <a href="#band">Band</a></code> |  |
| <code>stall_delay: <a href="#delayband">DelayBand</a></code> |  |
| <code>stall_rate: f64</code> |  |
| <code>tremor_axial_scale: f64</code> |  |
| <code>tremor_base: f64</code> |  |
| <code>tremor_hz: Vec&lt;f64&gt;</code> |  |
| <code>tremor_max: f64</code> |  |
| <code>tremor_per_px: f64</code> |  |
| <code>wait_for_click: <a href="#delayband">DelayBand</a></code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


### `DelayBand`

**Methods**

| Method | Description |
| --- | --- |
| <code>pick(self, rng: &amp;mut <a href="#rng">Rng</a>) -&gt; Duration</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>hi: Duration</code> |  |
| <code>lo: Duration</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Keystroke`

**Fields**

| Field | Description |
| --- | --- |
| <code>before: Duration</code> |  |
| <code>key: <a href="#key">Key</a></code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Point`

**Fields**

| Field | Description |
| --- | --- |
| <code>x: f64</code> |  |
| <code>y: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Rect`

**Fields**

| Field | Description |
| --- | --- |
| <code>height: f64</code> |  |
| <code>width: f64</code> |  |
| <code>x: f64</code> |  |
| <code>y: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Rng`

**Methods**

| Method | Description |
| --- | --- |
| <code>chance(&amp;mut self, p: f64) -&gt; bool</code> |  |
| <code>from_entropy() -&gt; Self</code> |  |
| <code>gauss(&amp;mut self) -&gt; f64</code> |  |
| <code>new(seed: u64) -&gt; Self</code> |  |
| <code>next_f64(&amp;mut self) -&gt; f64</code> |  |
| <code>next_u64(&amp;mut self) -&gt; u64</code> |  |
| <code>range(&amp;mut self, lo: f64, hi: f64) -&gt; f64</code> |  |
| <code>range_int(&amp;mut self, lo: u64, hi: u64) -&gt; u64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Debug</code>


### `Step`

**Fields**

| Field | Description |
| --- | --- |
| <code>after: Duration</code> |  |
| <code>point: <a href="#point">Point</a></code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `TypingOptions`

**Methods**

| Method | Description |
| --- | --- |
| <code>ensure_correction(self) -&gt; Self</code> |  |
| <code>typo_free() -&gt; Self</code> |  |

**Fields**

| Field | Description |
| --- | --- |
| <code>backspace_delay: Duration</code> |  |
| <code>backspace_scale_extra: f64</code> |  |
| <code>backspace_scale_min: f64</code> |  |
| <code>bigram_boost: f64</code> |  |
| <code>burst_factor: f64</code> |  |
| <code>burst_max_len: u64</code> |  |
| <code>burst_min_len: u64</code> |  |
| <code>burst_rate: f64</code> |  |
| <code>chars_per_word: f64</code> |  |
| <code>close_keys_boost: f64</code> |  |
| <code>close_keys_distance: usize</code> |  |
| <code>drift_momentum: f64</code> |  |
| <code>drift_range: f64</code> |  |
| <code>drift_volatility: f64</code> |  |
| <code>error_detection_rate: f64</code> |  |
| <code>error_rate: f64</code> |  |
| <code>far_keys_distance: usize</code> |  |
| <code>far_keys_penalty: f64</code> |  |
| <code>fast_bigrams: Vec&lt;&amp;'static str&gt;</code> |  |
| <code>fatigue_cap: f64</code> |  |
| <code>fatigue_factor: f64</code> |  |
| <code>first_char_factor: f64</code> |  |
| <code>jitter_sd: f64</code> |  |
| <code>keyboard_rows: Vec&lt;&amp;'static str&gt;</code> |  |
| <code>max_char_delay: Duration</code> |  |
| <code>min_char_delay: Duration</code> |  |
| <code>min_word_pause: Duration</code> |  |
| <code>reaction_scale_extra: f64</code> |  |
| <code>reaction_scale_min: f64</code> |  |
| <code>reaction_time: Duration</code> |  |
| <code>swap_error_rate: f64</code> |  |
| <code>thinking_pause_extra: Duration</code> |  |
| <code>thinking_pause_min: Duration</code> |  |
| <code>thinking_pause_rate: f64</code> |  |
| <code>uppercase_penalty: Duration</code> |  |
| <code>word_pause: Duration</code> |  |
| <code>word_pause_extra: Duration</code> |  |
| <code>word_pause_variance: Duration</code> |  |
| <code>wpm: f64</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Default</code>, <code>Debug</code>, <code>StructuralPartialEq</code>


## Enums

### `Key`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Backspace</code> |  |
| <code>Char(char)</code> |  |
| <code>Enter</code> |  |
| <code>Space</code> |  |
| <code>Tab</code> |  |

**Trait implementations:** <code>Clone</code>, <code>Eq</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


### `Target`

**Variants**

| Variant | Description |
| --- | --- |
| <code>Point(<a href="#point">Point</a>)</code> |  |
| <code>Rect(<a href="#rect">Rect</a>)</code> |  |

**Trait implementations:** <code>Clone</code>, <code>PartialEq</code>, <code>Debug</code>, <code>Copy</code>, <code>StructuralPartialEq</code>


## Functions

### `keystrokes`

<pre>pub fn <a href="#keystrokes">keystrokes</a>(text: &amp;str, options: &amp;<a href="#typingoptions">TypingOptions</a>, rng: &amp;mut <a href="#rng">Rng</a>) -&gt; Vec&lt;<a href="#keystroke">Keystroke</a>&gt;</pre>


### `path`

<pre>pub fn <a href="#path">path</a>(from: <a href="#point">Point</a>, to: <a href="#target">Target</a>, options: &amp;<a href="#cursoroptions">CursorOptions</a>, rng: &amp;mut <a href="#rng">Rng</a>) -&gt; Vec&lt;<a href="#step">Step</a>&gt;</pre>


### `target_point`

<pre>pub fn <a href="#target_point">target_point</a>(rect: <a href="#rect">Rect</a>, padding_percentage: f64, options: &amp;<a href="#cursoroptions">CursorOptions</a>, rng: &amp;mut <a href="#rng">Rng</a>) -&gt; <a href="#point">Point</a></pre>


---

Generated by truesight from `charm-cdp` 0.1.0. See the [API overview](index.md).
