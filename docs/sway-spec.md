# Sway 功能规格与 rway 兼容性分析

> 本文档基于 sway(5), sway-ipc(7), sway-input(5), sway-output(5), sway-bar(5) man pages 编写。
> 每个功能点标注实现状态：`[IMPLEMENTED]`, `[PARTIAL]`, `[MISSING]`
> 优先级：`P0`（基本可用）, `P1`（功能完整）, `P2`（增强）

---

## 1. 配置指令

### 1.1 通用配置 (General)

| 指令 | 语法 | 默认值 | 状态 | 优先级 |
|------|------|--------|------|--------|
| `set` | `set $<name> <value>` | - | `[IMPLEMENTED]` | P0 |
| `include` | `include <paths...>` | - | `[IMPLEMENTED]` | P1 |
| `font` | `font [pango:]<font>` | `monospace 10` | `[IMPLEMENTED]` | P1 |
| `default_orientation` | `default_orientation horizontal\|vertical\|auto` | auto | `[IMPLEMENTED]` | P1 |
| `workspace_layout` | `workspace_layout default\|stacking\|tabbed` | default | `[IMPLEMENTED]` | P1 |
| `xwayland` | `xwayland enable\|disable\|force` | enable | `[IMPLEMENTED]` | P1 |
| `swaybg_command` | `swaybg_command <command>` | `swaybg` | `[IMPLEMENTED]` | P2 |
| `swaynag_command` | `swaynag_command <command>` | `swaynag` | `[IMPLEMENTED]` | P2 |
| `floating_modifier` | `floating_modifier <modifier> [normal\|inverse]` | - | `[IMPLEMENTED]` | P0 |
| `floating_maximum_size` | `floating_maximum_size <w> x <h>` | `0 x 0` (unlimited) | `[IMPLEMENTED]` | P1 |
| `floating_minimum_size` | `floating_minimum_size <w> x <h>` | `75 x 50` | `[IMPLEMENTED]` | P1 |
| `focus_follows_mouse` | `focus_follows_mouse yes\|no\|always` | no | `[IMPLEMENTED]` | P0 |
| `focus_on_window_activation` | `focus_on_window_activation smart\|urgent\|focus\|none` | `urgent` | `[IMPLEMENTED]` | P1 |
| `focus_wrapping` | `focus_wrapping yes\|no\|force\|workspace` | `yes` | `[IMPLEMENTED]` | P1 |
| `mouse_warping` | `mouse_warping output\|container\|none` | `output` | `[IMPLEMENTED]` | P1 |
| `popup_during_fullscreen` | `popup_during_fullscreen smart\|ignore\|leave_fullscreen` | `smart` | `[IMPLEMENTED]` | P2 |
| `primary_selection` | `primary_selection enabled\|disabled` | `enabled` | `[MISSING]` | P2 |
| `show_marks` | `show_marks yes\|no` | `yes` | `[IMPLEMENTED]` | P2 |
| `tiling_drag` | `tiling_drag enable\|disable\|toggle` | `enable` | `[IMPLEMENTED]` | P1 |
| `tiling_drag_threshold` | `tiling_drag_threshold <threshold>` | `9` | `[MISSING]` | P2 |
| `title_align` | `title_align left\|center\|right` | left | `[IMPLEMENTED]` | P2 |
| `workspace_auto_back_and_forth` | `workspace_auto_back_and_forth yes\|no` | `no` | `[IMPLEMENTED]` | P1 |

### 1.2 按键绑定 (Keybindings)

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `bindsym` | `bindsym [flags] <key combo> <command>` | `[IMPLEMENTED]` | P0 |
| `bindcode` | `bindcode [flags] <code> <command>` | `[IMPLEMENTED]` | P1 |
| `bindswitch` | `bindswitch [flags] <switch>:<state> <command>` | `[MISSING]` | P2 |
| `bindgesture` | `bindgesture [flags] <gesture>[:<fingers>][:dirs] <command>` | `[MISSING]` | P2 |
| `unbindsym` | `unbindsym [flags] <key combo>` | `[IMPLEMENTED]` | P1 |
| `unbindcode` | `unbindcode [flags] <code>` | `[MISSING]` | P2 |
| `unbindswitch` | `unbindswitch <switch>:<state>` | `[MISSING]` | P2 |
| `unbindgesture` | `unbindgesture [flags] <gesture>[:<fingers>]` | `[MISSING]` | P2 |
| `mode` | `mode [--pango_markup] <mode> [subcommands]` | `[IMPLEMENTED]` | P1 |

**bindsym 标志支持情况：**

| 标志 | 状态 | 优先级 |
|------|------|--------|
| `--release` | `[IMPLEMENTED]` | P1 |
| `--locked` | `[IMPLEMENTED]` | P1 |
| `--whole-window` | `[IMPLEMENTED]` | P2 |
| `--border` | `[IMPLEMENTED]` | P2 |
| `--exclude-titlebar` | `[MISSING]` | P2 |
| `--inhibited` | `[MISSING]` | P2 |
| `--no-warn` | `[MISSING]` | P2 |
| `--no-repeat` | `[IMPLEMENTED]` | P2 |
| `--input-device=<device>` | `[MISSING]` | P2 |

**修饰键支持情况：**

| 修饰键 | 状态 |
|--------|------|
| `Mod1` (Alt) | `[IMPLEMENTED]` |
| `Mod4` (Super) | `[IMPLEMENTED]` |
| `Shift` | `[IMPLEMENTED]` |
| `Control`/`Ctrl` | `[IMPLEMENTED]` |
| `Alt` | `[IMPLEMENTED]` |
| `Mod2` (Num Lock) | `[MISSING]` |
| `Mod3` | `[MISSING]` |
| `Mod5` | `[MISSING]` |

### 1.3 命令动作 (Commands)

#### 1.3.1 窗口管理

| 命令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `exec` | `exec <shell command>` | `[IMPLEMENTED]` | P0 |
| `exec_always` | `exec_always <shell command>` | `[IMPLEMENTED]` | P0 |
| `kill` | `kill` | `[IMPLEMENTED]` | P0 |
| `focus` direction | `focus up\|right\|down\|left` | `[IMPLEMENTED]` | P0 |
| `focus` sibling | `focus prev\|next [sibling]` | `[IMPLEMENTED]` | P1 |
| `focus` hierarchy | `focus child\|parent` | `[MISSING]` | P0 |
| `focus` output | `focus output <name>\|up\|right\|down\|left` | `[IMPLEMENTED]` | P1 |
| `focus` type | `focus tiling\|floating\|mode_toggle` | `[MISSING]` | P1 |
| `move` direction | `move left\|right\|up\|down [<px> px]` | `[IMPLEMENTED]` | P0 |
| `move` position | `move [absolute] position <x> <y>` | `[IMPLEMENTED]` | P1 |
| `move` center | `move position center\|cursor\|mouse\|pointer` | `[IMPLEMENTED]` | P1 |
| `move` to mark | `move [container\|window] to mark <mark>` | `[MISSING]` | P2 |
| `move` to workspace | `move [container\|window] to workspace <name>` | `[IMPLEMENTED]` | P0 |
| `move` to output | `move [container\|window] to output <name>` | `[IMPLEMENTED]` | P1 |
| `move` to scratchpad | `move [container\|window] to scratchpad` | `[IMPLEMENTED]` | P1 |
| `workspace` | `workspace [--no-auto-back-and-forth] [number] <name>` | `[IMPLEMENTED]` | P0 |
| `split` | `split vertical\|v\|horizontal\|h\|none\|toggle` | `[IMPLEMENTED]` | P0 |
| `splith` | `splith` | `[IMPLEMENTED]` | P0 |
| `splitv` | `splitv` | `[IMPLEMENTED]` | P0 |
| `splitt` | `splitt` | `[MISSING]` | P1 |
| `layout` | `layout default\|splith\|splitv\|stacking\|tabbed` | `[IMPLEMENTED]` | P0 |
| `layout toggle` | `layout toggle [split\|all]` | `[IMPLEMENTED]` | P1 |
| `floating` | `floating enable\|disable\|toggle` | `[IMPLEMENTED]` | P0 |
| `fullscreen` | `fullscreen [enable\|disable\|toggle] [global]` | `[IMPLEMENTED]` | P0 |
| `sticky` | `sticky enable\|disable\|toggle` | `[IMPLEMENTED]` | P2 |
| `resize` shrink/grow | `resize shrink\|grow width\|height [<amount> [px\|ppt]]` | `[IMPLEMENTED]` | P0 |
| `resize` set | `resize set [width] <w> [height] <h>` | `[IMPLEMENTED]` | P1 |
| `swap` | `swap container with id\|con_id\|mark <arg>` | `[IMPLEMENTED]` | P1 |
| `rename` | `rename workspace [<old>] to <new>` | `[IMPLEMENTED]` | P1 |
| `scratchpad show` | `scratchpad show` | `[IMPLEMENTED]` | P1 |
| `nop` | `nop <comment>` | `[IMPLEMENTED]` | P2 |
| `reload` | `reload` | `[IMPLEMENTED]` | P0 |
| `exit` | `exit` | `[IMPLEMENTED]` | P0 |

#### 1.3.2 外观

| 命令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `border` | `border none\|normal\|csd\|pixel [<n>]` | `[IMPLEMENTED]` | P1 |
| `border toggle` | `border toggle` | `[IMPLEMENTED]` | P2 |
| `default_border` | `default_border normal\|none\|pixel [<n>]` | `[IMPLEMENTED]` | P0 |
| `default_floating_border` | `default_floating_border normal\|none\|pixel [<n>]` | `[IMPLEMENTED]` | P1 |
| `gaps` | `gaps inner\|outer <amount>` | `[IMPLEMENTED]` | P0 |
| `hide_edge_borders` | `hide_edge_borders [--i3] none\|vertical\|horizontal\|both\|smart` | `[IMPLEMENTED]` | P1 |
| `smart_borders` | `smart_borders on\|no_gaps\|off` | `[IMPLEMENTED]` | P1 |
| `smart_gaps` | `smart_gaps on\|off\|toggle\|inverse_outer` | `[IMPLEMENTED]` | P1 |
| `opacity` | `opacity [set\|plus\|minus] <value>` | `[IMPLEMENTED]` | P2 |
| `title_format` | `title_format <format>` | `[IMPLEMENTED]` | P2 |
| `titlebar_border_thickness` | `titlebar_border_thickness <px>` | `[IMPLEMENTED]` | P2 |
| `titlebar_padding` | `titlebar_padding <h> [<v>]` | `[IMPLEMENTED]` | P2 |
| `max_render_time` | `max_render_time off\|<msec>` | `[IMPLEMENTED]` | P2 |
| `allow_tearing` | `allow_tearing yes\|no` | `[IMPLEMENTED]` | P2 |
| `inhibit_idle` | `inhibit_idle focus\|fullscreen\|open\|none\|visible` | `[MISSING]` | P2 |
| `shortcuts_inhibitor` | `shortcuts_inhibitor enable\|disable` | `[MISSING]` | P2 |

#### 1.3.3 标记与分配

| 命令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `mark` | `mark [--add\|--replace] [--toggle] <id>` | `[MISSING]` | P1 |
| `unmark` | `unmark [<identifier>]` | `[MISSING]` | P1 |
| `assign` | `assign <criteria> [->] [workspace\|output] <target>` | `[IMPLEMENTED]` | P1 |
| `no_focus` | `no_focus <criteria>` | `[IMPLEMENTED]` | P2 |
| `urgent` | `urgent enable\|disable\|allow\|deny` | `[IMPLEMENTED]` | P2 |
| `force_display_urgency_hint` | `force_display_urgency_hint <timeout> [ms]` | `[MISSING]` | P2 |

### 1.4 窗口条件 (Criteria)

| 条件属性 | 说明 | 状态 | 优先级 |
|----------|------|------|--------|
| `app_id` | Wayland app_id | `[IMPLEMENTED]` | P0 |
| `class` | X11 WM_CLASS | `[IMPLEMENTED]` | P0 |
| `title` | 窗口标题 | `[IMPLEMENTED]` | P0 |
| `con_id` | 容器 ID | `[MISSING]` | P1 |
| `con_mark` | 标记 | `[MISSING]` | P1 |
| `floating` | 浮动状态 | `[IMPLEMENTED]` | P1 |
| `tiling` | 平铺状态 | `[IMPLEMENTED]` | P1 |
| `id` | X11 window ID | `[MISSING]` | P2 |
| `instance` | X11 instance | `[MISSING]` | P2 |
| `pid` | 进程 ID | `[MISSING]` | P1 |
| `shell` | shell 类型 (xdg_shell\|xwayland) | `[MISSING]` | P1 |
| `urgent` | 紧急状态 | `[IMPLEMENTED]` | P2 |
| `window_role` | X11 window role | `[MISSING]` | P2 |
| `window_type` | X11 window type | `[IMPLEMENTED]` | P2 |
| `workspace` | 所在工作区 | `[IMPLEMENTED]` | P1 |
| `all` | 匹配所有 | `[IMPLEMENTED]` | P2 |

### 1.5 窗口规则 (for_window)

| 动作 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `for_window` | `for_window <criteria> <command>` | `[IMPLEMENTED]` | P0 |
| 支持的动作: `floating enable/disable` | | `[IMPLEMENTED]` | P0 |
| 支持的动作: `move to workspace` | | `[IMPLEMENTED]` | P0 |
| 支持的动作: `resize` | | `[IMPLEMENTED]` | P1 |
| 支持任意命令作为动作 | | `[MISSING]` | P1 |

### 1.6 颜色配置 (Colors)

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `client.focused` | `client.focused <border> <bg> <text> [<indicator>] [<child_border>]` | `[MISSING]` | P1 |
| `client.focused_inactive` | 同上 | `[MISSING]` | P1 |
| `client.unfocused` | 同上 | `[MISSING]` | P1 |
| `client.urgent` | 同上 | `[MISSING]` | P1 |
| `client.placeholder` | 同上 | `[MISSING]` | P2 |
| `client.background` | `client.background <color>` | `[MISSING]` | P2 |

---

## 2. 输入配置 (sway-input)

### 2.1 键盘配置

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `input <id> xkb_layout` | `input <id> xkb_layout <layout>` | `[IMPLEMENTED]` | P0 |
| `input <id> xkb_variant` | `input <id> xkb_variant <variant>` | `[IMPLEMENTED]` | P1 |
| `input <id> xkb_model` | `input <id> xkb_model <model>` | `[IMPLEMENTED]` | P1 |
| `input <id> xkb_options` | `input <id> xkb_options <options>` | `[IMPLEMENTED]` | P0 |
| `input <id> xkb_rules` | `input <id> xkb_rules <rules>` | `[MISSING]` | P2 |
| `input <id> xkb_file` | `input <id> xkb_file <file>` | `[MISSING]` | P2 |
| `input <id> xkb_switch_layout` | `input <id> xkb_switch_layout <idx>\|next\|prev` | `[MISSING]` | P1 |
| `input <id> xkb_capslock` | `input <id> xkb_capslock enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> xkb_numlock` | `input <id> xkb_numlock enabled\|disabled` | `[MISSING]` | P1 |
| `input <id> repeat_delay` | `input <id> repeat_delay <ms>` | `[IMPLEMENTED]` | P1 |
| `input <id> repeat_rate` | `input <id> repeat_rate <cps>` | `[IMPLEMENTED]` | P1 |

### 2.2 指针/触控板配置

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `input <id> accel_profile` | `adaptive\|flat` | `[IMPLEMENTED]` | P1 |
| `input <id> pointer_accel` | `<-1..1>` | `[IMPLEMENTED]` | P1 |
| `input <id> natural_scroll` | `enabled\|disabled` | `[IMPLEMENTED]` | P1 |
| `input <id> scroll_method` | `none\|two_finger\|edge\|on_button_down` | `[IMPLEMENTED]` | P1 |
| `input <id> scroll_button` | `disable\|button[1-3,8,9]\|<code>` | `[MISSING]` | P2 |
| `input <id> scroll_button_lock` | `enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> scroll_factor` | `<float>` | `[MISSING]` | P1 |
| `input <id> tap` | `enabled\|disabled` | `[IMPLEMENTED]` | P1 |
| `input <id> tap_button_map` | `lrm\|lmr` | `[MISSING]` | P2 |
| `input <id> drag` | `enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> drag_lock` | `enabled\|disabled\|enabled_sticky` | `[MISSING]` | P2 |
| `input <id> dwt` | `enabled\|disabled` | `[IMPLEMENTED]` | P1 |
| `input <id> dwtp` | `enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> left_handed` | `enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> middle_emulation` | `enabled\|disabled` | `[MISSING]` | P2 |
| `input <id> click_method` | `none\|button_areas\|clickfinger` | `[MISSING]` | P2 |
| `input <id> clickfinger_button_map` | `lrm\|lmr` | `[MISSING]` | P2 |
| `input <id> rotation_angle` | `<0.0..360.0>` | `[MISSING]` | P2 |
| `input <id> events` | `enabled\|disabled\|disabled_on_external_mouse\|toggle` | `[MISSING]` | P2 |
| `input <id> calibration_matrix` | `<6 floats>` | `[MISSING]` | P2 |

### 2.3 映射配置

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `input <id> map_to_output` | `<output\|*>` | `[MISSING]` | P2 |
| `input <id> map_to_region` | `<X> <Y> <W> <H>` | `[MISSING]` | P2 |
| `input <id> map_from_region` | `<X1xY1> <X2xY2>` | `[MISSING]` | P2 |
| `input <id> tool_mode` | `<tool> <absolute\|relative>` | `[MISSING]` | P2 |

### 2.4 Seat 配置

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `seat <name> attach` | `seat <name> attach <input_id>` | `[MISSING]` | P2 |
| `seat <name> fallback` | `seat <name> fallback true\|false` | `[MISSING]` | P2 |
| `seat <name> hide_cursor` | `seat <name> hide_cursor <timeout>\|when-typing [enable\|disable]` | `[MISSING]` | P1 |
| `seat <name> idle_inhibit` | `seat <name> idle_inhibit <sources...>` | `[MISSING]` | P2 |
| `seat <name> keyboard_grouping` | `seat <name> keyboard_grouping none\|smart` | `[MISSING]` | P2 |
| `seat <name> pointer_constraint` | `seat <name> pointer_constraint enable\|disable\|escape` | `[MISSING]` | P2 |
| `seat <name> shortcuts_inhibitor` | `enable\|disable\|activate\|deactivate\|toggle` | `[MISSING]` | P2 |
| `seat <name> xcursor_theme` | `seat <name> xcursor_theme <theme> [<size>]` | `[MISSING]` | P1 |

---

## 3. 输出配置 (sway-output)

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `output <name> mode` | `mode\|resolution\|res [--custom] <W>x<H>[@<Hz>]` | `[IMPLEMENTED]` | P0 |
| `output <name> position` | `position\|pos <X> <Y>` | `[IMPLEMENTED]` | P0 |
| `output <name> scale` | `scale <factor>` | `[IMPLEMENTED]` | P0 |
| `output <name> scale_filter` | `scale_filter linear\|nearest\|smart` | `[MISSING]` | P2 |
| `output <name> transform` | `transform <transform> [clockwise\|anticlockwise]` | `[IMPLEMENTED]` | P1 |
| `output <name> enable/disable` | `enable\|disable` | `[MISSING]` | P1 |
| `output <name> toggle` | `toggle` | `[MISSING]` | P2 |
| `output <name> power` | `power on\|off\|toggle` | `[MISSING]` | P1 |
| `output <name> dpms` | `dpms on\|off\|toggle` (deprecated) | `[MISSING]` | P2 |
| `output <name> background` | `bg <file> <mode> [<fallback_color>]` | `[MISSING]` | P1 |
| `output <name> background` | `bg <color> solid_color` | `[MISSING]` | P1 |
| `output <name> subpixel` | `subpixel rgb\|bgr\|vrgb\|vbgr\|none` | `[MISSING]` | P2 |
| `output <name> adaptive_sync` | `adaptive_sync on\|off\|toggle` | `[MISSING]` | P2 |
| `output <name> render_bit_depth` | `render_bit_depth 6\|8\|10` | `[MISSING]` | P2 |
| `output <name> max_render_time` | `max_render_time off\|<msec>` | `[MISSING]` | P2 |
| `output <name> color_profile` | `color_profile srgb\|icc <file>` | `[MISSING]` | P2 |
| `output <name> allow_tearing` | `allow_tearing yes\|no` | `[MISSING]` | P2 |
| `output <name> modeline` | `modeline <params...>` | `[MISSING]` | P2 |

---

## 4. Bar 配置 (sway-bar)

| 指令 | 语法 | 状态 | 优先级 |
|------|------|------|--------|
| `bar { }` | 配置块 | `[IMPLEMENTED]` | P1 |
| `id` | `id <bar_id>` | `[MISSING]` | P1 |
| `swaybar_command` | `swaybar_command <command>` | `[MISSING]` | P1 |
| `status_command` | `status_command <command>` | `[IMPLEMENTED]` | P1 |
| `mode` | `mode dock\|hide\|invisible\|overlay` | `[IMPLEMENTED]` | P1 |
| `hidden_state` | `hidden_state hide\|show` | `[MISSING]` | P2 |
| `position` | `position top\|bottom` | `[IMPLEMENTED]` | P1 |
| `output` | `output <output>\|\*` | `[IMPLEMENTED]` | P2 |
| `font` | `font <font>` | `[IMPLEMENTED]` | P2 |
| `separator_symbol` | `separator_symbol <symbol>` | `[MISSING]` | P2 |
| `wrap_scroll` | `wrap_scroll yes\|no` | `[MISSING]` | P2 |
| `workspace_buttons` | `workspace_buttons yes\|no` | `[MISSING]` | P1 |
| `workspace_min_width` | `workspace_min_width <px>` | `[MISSING]` | P2 |
| `strip_workspace_numbers` | `strip_workspace_numbers yes\|no` | `[MISSING]` | P2 |
| `strip_workspace_name` | `strip_workspace_name yes\|no` | `[MISSING]` | P2 |
| `binding_mode_indicator` | `binding_mode_indicator yes\|no` | `[MISSING]` | P2 |
| `modifier` | `modifier <Modifier>\|none` | `[IMPLEMENTED]` | P2 |
| `pango_markup` | `pango_markup enabled\|disabled` | `[MISSING]` | P2 |
| `height` | `height <height>` | `[MISSING]` | P2 |
| `gaps` | `gaps <all>\|<horiz> <vert>\|<top> <right> <bottom> <left>` | `[IMPLEMENTED]` | P2 |
| `status_padding` | `status_padding <padding>` | `[MISSING]` | P2 |
| `status_edge_padding` | `status_edge_padding <padding>` | `[MISSING]` | P2 |
| `colors { }` | 颜色配置块 | `[IMPLEMENTED]` | P1 |
| tray 系列指令 | `tray_bindcode`, `tray_bindsym`, `tray_padding`, `tray_output`, `icon_theme` | `[MISSING]` | P2 |

---

## 5. IPC 协议 (sway-ipc)

### 5.1 消息类型

| 类型码 | 名称 | 状态 | 优先级 |
|--------|------|------|--------|
| 0 | `RUN_COMMAND` | `[IMPLEMENTED]` | P0 |
| 1 | `GET_WORKSPACES` | `[IMPLEMENTED]` | P0 |
| 2 | `SUBSCRIBE` | `[IMPLEMENTED]` | P0 |
| 3 | `GET_OUTPUTS` | `[IMPLEMENTED]` | P0 |
| 4 | `GET_TREE` | `[IMPLEMENTED]` | P0 |
| 5 | `GET_MARKS` | `[IMPLEMENTED]` | P1 |
| 6 | `GET_BAR_CONFIG` | `[IMPLEMENTED]` | P1 |
| 7 | `GET_VERSION` | `[IMPLEMENTED]` | P0 |
| 8 | `GET_BINDING_MODES` | `[IMPLEMENTED]` | P1 |
| 9 | `GET_CONFIG` | `[IMPLEMENTED]` | P1 |
| 10 | `SEND_TICK` | `[IMPLEMENTED]` | P2 |
| 11 | `SYNC` | `[IMPLEMENTED]` | P2 |
| 12 | `GET_BINDING_STATE` | `[MISSING]` | P1 |
| 100 | `GET_INPUTS` | `[IMPLEMENTED]` | P1 |
| 101 | `GET_SEATS` | `[IMPLEMENTED]` | P1 |

### 5.2 RUN_COMMAND (类型 0) 详细规格

**请求:** 命令字符串（可以用分号分隔多个命令）

**响应格式:**
```json
[{"success": true|false, "error": "string (optional)", "parse_error": true|false (optional)}]
```

**rway 当前状态:** `[PARTIAL]` — 接收命令但不实际解析执行，固定返回 `[{"success": true}]`

### 5.3 GET_WORKSPACES (类型 1) 详细规格

**响应格式:**
```json
[{
  "num": 1,
  "name": "1",
  "visible": true,
  "focused": true,
  "urgent": false,
  "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080},
  "output": "HDMI-A-1"
}]
```

**rway 当前状态:** `[IMPLEMENTED]` — 字段完整，但 output 硬编码为 "winit"

### 5.4 GET_OUTPUTS (类型 3) 详细规格

**Sway 完整响应字段:**
```json
{
  "name": "string",
  "make": "string",
  "model": "string",
  "serial": "string",
  "active": true,
  "dpms": true,
  "power": true,
  "primary": false,
  "scale": 1.0,
  "subpixel_hinting": "rgb",
  "transform": "normal",
  "current_workspace": "1",
  "modes": [{"width": 1920, "height": 1080, "refresh": 60000}],
  "current_mode": {"width": 1920, "height": 1080, "refresh": 60000},
  "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}
}
```

**rway 缺失字段:** `dpms`, `power`, `subpixel_hinting`, `modes` (只有 current_mode)

### 5.5 GET_TREE (类型 4) 详细规格

**Sway 完整 TreeNode 字段:**
```json
{
  "id": 1,
  "name": "string",
  "type": "root|output|workspace|con|floating_con",
  "border": "normal|none|pixel|csd",
  "current_border_width": 2,
  "layout": "splith|splitv|stacked|tabbed|output|none",
  "orientation": "vertical|horizontal|none",
  "percent": 0.5,
  "rect": {},
  "window_rect": {},
  "deco_rect": {},
  "geometry": {},
  "urgent": false,
  "sticky": false,
  "marks": [],
  "focused": false,
  "focus": [2, 3],
  "nodes": [],
  "floating_nodes": [],
  "representation": "H[V[...]]",
  "fullscreen_mode": 0,
  "floating": "auto_off",
  "scratchpad_state": "none",
  "app_id": "string",
  "pid": 12345,
  "visible": true,
  "shell": "xdg_shell",
  "inhibit_idle": false,
  "idle_inhibitors": {"application": "none", "user": "none"},
  "window": null,
  "window_properties": {}
}
```

**rway 缺失字段:** `border`, `current_border_width`, `orientation`, `percent`, `sticky`, `marks`, `representation`, `fullscreen_mode`, `floating` (string), `scratchpad_state`, `pid`, `visible`, `shell`, `inhibit_idle`, `idle_inhibitors`, `window_properties`

### 5.6 事件类型

| 事件码 | 名称 | Payload | 状态 | 优先级 |
|--------|------|---------|------|--------|
| 0x80000000 | `workspace` | `{change, current, old}` | `[PARTIAL]` | P0 |
| 0x80000001 | `output` | `{change}` | `[IMPLEMENTED]` | P1 |
| 0x80000002 | `mode` | `{change, pango_markup}` | `[IMPLEMENTED]` | P1 |
| 0x80000003 | `window` | `{change, container}` | `[IMPLEMENTED]` | P0 |
| 0x80000004 | `barconfig_update` | bar config object | `[MISSING]` | P2 |
| 0x80000005 | `binding` | `{change, binding}` | `[IMPLEMENTED]` | P1 |
| 0x80000006 | `shutdown` | `{change}` | `[IMPLEMENTED]` | P1 |
| 0x80000007 | `tick` | `{first, payload}` | `[IMPLEMENTED]` | P2 |
| 0x80000014 | `bar_state_update` | `{id, visible_by_modifier}` | `[IMPLEMENTED]` | P2 |
| 0x80000015 | `input` | `{change, input}` | `[IMPLEMENTED]` | P2 |

**workspace 事件 change 值:** `init`, `empty`, `focus`, `move`, `rename`, `urgent`, `reload`

**window 事件 change 值:** `new`, `close`, `focus`, `title`, `fullscreen_mode`, `move`, `floating`, `urgent`, `mark`

---

## 6. 窗口管理行为规格

### 6.1 Focus 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| 方向焦点 | `focus left/right/up/down` — 在同一容器内按布局方向移动焦点 | `[IMPLEMENTED]` | P0 |
| 父/子焦点 | `focus parent` 上移到父容器，`focus child` 下移到子容器 | `[IMPLEMENTED]` | P0 |
| 兄弟焦点 | `focus prev/next [sibling]` 在同级容器间切换 | `[MISSING]` | P1 |
| 跨输出焦点 | `focus output <name/direction>` 移动焦点到其他输出 | `[IMPLEMENTED]` | P1 |
| 浮动/平铺切换 | `focus tiling/floating/mode_toggle` 在两种类型间切换焦点 | `[IMPLEMENTED]` | P1 |
| focus_follows_mouse | 鼠标移入窗口时自动聚焦；`yes`=新窗口保留焦点, `always`=总是跟随 | `[IMPLEMENTED]` | P0 |
| focus_wrapping | `yes`=容器边界回绕, `no`=停止, `force`=强制回绕, `workspace`=工作区边界回绕 | `[IMPLEMENTED]` | P1 |
| 自动聚焦新窗口 | 新创建的顶层窗口自动获取焦点 | `[IMPLEMENTED]` | P0 |

**Sway focus 精确行为:**
1. `focus left/right` 在 SplitH 容器内切换 `focused_child`
2. `focus up/down` 在 SplitV 容器内切换 `focused_child`
3. 如果当前方向与容器布局不匹配，向上遍历父容器直到找到匹配的
4. Tabbed/Stacked 容器中 `focus left/right` 等同于 `focus prev/next`
5. 到达容器边界时，如果 `focus_wrapping=yes`，继续到兄弟容器中查找

### 6.2 Move 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| 方向移动 | `move left/right/up/down [<px> px]` — 平铺窗口在容器内交换位置，浮动窗口按像素移动 | `[IMPLEMENTED]` | P0 |
| 跨容器移动 | 当窗口在容器边界移动时，提升到父容器或移入相邻容器 | `[MISSING]` | P0 |
| 跨工作区移动 | `move container to workspace <name>` | `[IMPLEMENTED]` | P0 |
| 跨输出移动 | `move container to output <name/direction>` | `[IMPLEMENTED]` | P1 |
| 绝对定位 | `move [absolute] position <x> <y>` (仅浮动窗口) | `[IMPLEMENTED]` | P1 |
| 居中定位 | `move position center` (仅浮动窗口) | `[IMPLEMENTED]` | P1 |
| Scratchpad | `move container to scratchpad` | `[IMPLEMENTED]` | P1 |

**Sway move 精确行为:**
1. 在 SplitH 中 `move left/right`：与相邻兄弟交换位置
2. 在 SplitH 中 `move up/down`：向上遍历找到 SplitV 父容器，将窗口提升到该层
3. 在容器边界 `move`：将窗口从当前容器移出到父容器的相邻位置
4. 跨输出边界移动：窗口转移到相邻输出的活跃工作区
5. 浮动窗口 `move`：按像素移动（默认 10px）

### 6.3 Resize 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| grow/shrink | `resize grow/shrink width/height <amount> [px\|ppt]` | `[IMPLEMENTED]` | P0 |
| resize set | `resize set [width] <w> [height] <h>` | `[IMPLEMENTED]` | P1 |
| 鼠标拖拽调整 | 通过 floating_modifier + 右键拖拽 | `[MISSING]` | P0 |

**Sway resize 精确行为:**
1. `ppt`（percentage points）：调整容器在其父级中的比例，默认单位
2. `px`（pixels）：调整像素大小
3. 默认增量：10 ppt 或 10 px
4. 平铺窗口调整影响相邻窗口的大小（比例重新分配）
5. 浮动窗口调整直接改变几何尺寸

### 6.4 Fullscreen 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| toggle | `fullscreen toggle` — 在全屏和正常之间切换 | `[IMPLEMENTED]` | P0 |
| enable/disable | `fullscreen enable/disable` — 显式设置 | `[IMPLEMENTED]` | P0 |
| global | `fullscreen toggle global` — 跨所有输出全屏 | `[IMPLEMENTED]` | P1 |

**Sway fullscreen 精确行为:**
- `fullscreen_mode=0`：正常
- `fullscreen_mode=1`：工作区内全屏（覆盖同工作区其他窗口）
- `fullscreen_mode=2`：全局全屏（覆盖所有输出）
- 全屏时保存原始 geometry，退出全屏时恢复

### 6.5 Floating 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| toggle | `floating toggle` | `[IMPLEMENTED]` | P0 |
| enable/disable | `floating enable/disable` | `[IMPLEMENTED]` | P0 |
| 默认尺寸 | 浮动窗口默认为原始请求尺寸或屏幕 75% | `[MISSING]` | P1 |
| 最大/最小约束 | `floating_maximum_size` / `floating_minimum_size` | `[IMPLEMENTED]` | P1 |
| floating_modifier | 修饰键 + 左键拖拽移动，右键拖拽调整大小 | `[IMPLEMENTED]` | P0 |

**Sway floating 精确行为:**
1. `floating enable`：从平铺树中移除，放入 `floating_nodes` 列表
2. 浮动窗口居中于其原来所在的工作区
3. 浮动窗口保持在平铺窗口之上
4. `floating_modifier` + 左键 = 拖拽移动，右键 = 拖拽调整大小
5. `inverse` 模式反转左右键行为

### 6.6 Scratchpad 行为

| 行为 | 描述 | 状态 | 优先级 |
|------|------|------|--------|
| `move to scratchpad` | 隐藏窗口到 scratchpad | `[IMPLEMENTED]` | P1 |
| `scratchpad show` | 显示/循环 scratchpad 窗口 | `[IMPLEMENTED]` | P1 |

**Sway scratchpad 精确行为:**
1. Scratchpad 是一个隐藏的窗口存储区
2. `move to scratchpad`：窗口变为浮动并隐藏
3. `scratchpad show`：如果有隐藏的 scratchpad 窗口，显示第一个；如果当前焦点在 scratchpad 窗口上，隐藏它；如果有多个，循环显示
4. Scratchpad 窗口在 IPC tree 中的 `scratchpad_state` 字段反映状态

---

## 7. 平铺布局规格

### 7.1 SplitH (水平分割)

**计算逻辑:**
1. 总可用宽度按 `sizes[]` 数组中各元素的比例分配
2. 每个子节点得到 `width * (sizes[i] / sum(sizes))` 的宽度
3. 最后一个子节点使用剩余宽度（避免浮点舍入误差）
4. 所有子节点高度等于父容器高度

**rway 状态:** `[IMPLEMENTED]` — 完整实现，包含比例分配和舍入处理

### 7.2 SplitV (垂直分割)

**计算逻辑:**
1. 总可用高度按 `sizes[]` 数组中各元素的比例分配
2. 每个子节点得到 `height * (sizes[i] / sum(sizes))` 的高度
3. 最后一个子节点使用剩余高度
4. 所有子节点宽度等于父容器宽度

**rway 状态:** `[IMPLEMENTED]` — 完整实现

### 7.3 Tabbed 布局

**行为:**
1. 所有子节点共享同一区域
2. 只有 `focused_child` 可见，其他子节点不渲染
3. 标题栏（tab bar）显示在区域顶部，列出所有子节点标题
4. 切换标签使用 `focus left/right` 或直接点击 tab
5. Tab bar 高度由 `font` 设置决定

**rway 状态:** `[PARTIAL]` — 布局计算正确（只有聚焦子节点获得完整区域），但缺少 tab bar 渲染

### 7.4 Stacked 布局

**行为:**
1. 与 Tabbed 类似，只有 `focused_child` 可见
2. 区别：标题栏纵向堆叠而非横向排列
3. 每个子节点的标题占一行
4. 切换使用 `focus up/down`

**rway 状态:** `[PARTIAL]` — 布局计算正确，但缺少 stacked title bar 渲染

### 7.5 Gaps 计算

| 类型 | 描述 | 状态 |
|------|------|------|
| `gaps inner` | 相邻窗口之间的间距 | `[IMPLEMENTED]` |
| `gaps outer` | 窗口与屏幕边缘的间距 | `[IMPLEMENTED]` |
| per-edge outer | `gaps outer top/right/bottom/left` — 各边独立设置 | `[IMPLEMENTED]` |
| `smart_gaps` | 单窗口时禁用 gaps | `[IMPLEMENTED]` |

**Sway gaps 精确计算:**
1. Outer gaps：在 Workspace 层级将可用区域四边各收缩 `outer` 像素
2. Inner gaps：在 Window（叶子）层级将可用区域四边各收缩 `inner/2` 像素
3. 相邻两窗口之间的实际间距 = `inner/2 + inner/2 = inner`
4. 窗口与屏幕边缘的实际间距 = `outer + inner/2`
5. Per-edge outer gaps 允许 `gaps outer top 10` 等单独设置

**rway 状态:**
- `[IMPLEMENTED]`: inner/outer 基本计算逻辑
- `[MISSING]`: per-edge outer gaps, smart_gaps, runtime `gaps` 命令调整

---

## 8. Gap Analysis（差距分析）

### P0 — 基本可用（阻塞核心使用）

| # | 功能 | 当前状态 | 差距描述 |
|---|------|----------|----------|
| 1 | **resize 命令** | MISSING | 无法通过快捷键/命令调整窗口大小，核心功能缺失 |
| 2 | **move 方向命令** | MISSING | 无法移动窗口位置（swap/reorder），已定义 Action::Move 但 `execute_action` 中标记为 TODO |
| 3 | **move to workspace** | MISSING | 无法将窗口移动到其他工作区，Action::MoveToWorkspace 标记为 TODO |
| 4 | **focus parent/child** | MISSING | 无法在容器层级间导航焦点 |
| 5 | **floating_modifier** | MISSING | 无法用修饰键+鼠标拖拽移动/调整浮动窗口 |
| 6 | **IPC SUBSCRIBE** | MISSING | waybar 等工具依赖事件订阅来实时更新，协议类型已定义但服务端未实现事件推送 |
| 7 | **IPC RUN_COMMAND 实际执行** | PARTIAL | 当前固定返回 success，不解析和执行命令内容 |
| 8 | **fullscreen enable/disable** | PARTIAL | 只有 toggle，无法显式 enable/disable |
| 9 | **floating enable/disable** | PARTIAL | 只有 toggle，无法显式 enable/disable |
| 10 | **focus_follows_mouse `always` 模式** | PARTIAL | 只支持 bool yes/no，不支持 `always` 三态 |

### P1 — 功能完整

| # | 功能 | 当前状态 | 差距描述 |
|---|------|----------|----------|
| 11 | **binding modes** | MISSING | 无 mode 支持，无法实现 resize mode 等常见模式 |
| 12 | **include 指令** | MISSING | 无法拆分配置文件 |
| 13 | **scratchpad** | MISSING | 无 scratchpad 功能 |
| 14 | **marks** | MISSING | 无 mark/unmark/goto mark 支持 |
| 15 | **assign** | MISSING | 无法自动将特定应用分配到工作区 |
| 16 | **bar 配置** | MISSING | 整个 bar 子系统缺失，swaybar/waybar 的 bar config IPC 不可用 |
| 17 | **客户端颜色** | MISSING | client.focused 等颜色配置全部缺失 |
| 18 | **border 运行时命令** | MISSING | 无法运行时修改窗口边框 |
| 19 | **hide_edge_borders / smart_borders** | MISSING | 单窗口时边框优化 |
| 20 | **跨输出焦点/移动** | MISSING | 多显示器间的窗口和焦点移动 |
| 21 | **focus_wrapping** | MISSING | 焦点到达容器边界时的回绕行为 |
| 22 | **键盘 XKB 配置** | MISSING | xkb_layout, xkb_options 等键盘布局配置 |
| 23 | **输入设备 libinput 配置** | MISSING | 触控板 tap, natural_scroll 等常用设置 |
| 24 | **GET_BAR_CONFIG** | MISSING | IPC 类型 6 |
| 25 | **GET_BINDING_MODES** | MISSING | IPC 类型 8 |
| 26 | **GET_CONFIG** | MISSING | IPC 类型 9 |
| 27 | **GET_BINDING_STATE** | MISSING | IPC 类型 12 |
| 28 | **IPC 事件推送** | MISSING | workspace/window 事件类型已定义但未集成到服务端 |
| 29 | **GET_TREE 字段不完整** | PARTIAL | 缺少 border, percent, marks, fullscreen_mode, shell, pid 等 16 个字段 |
| 30 | **GET_OUTPUTS 字段不完整** | PARTIAL | 缺少 dpms, power, subpixel_hinting, modes 列表 |
| 31 | **tabbed/stacked title bar** | PARTIAL | 布局正确但无可视 tab/stack 标题渲染 |
| 32 | **for_window 支持任意命令** | PARTIAL | 目前只支持 floating/move/resize 三种动作 |
| 33 | **workspace_auto_back_and_forth** | MISSING | 再次按当前工作区快捷键回到前一个 |
| 34 | **output background** | MISSING | 壁纸/背景色设置 |
| 35 | **output enable/disable/power** | MISSING | 输出管理命令 |
| 36 | **seat xcursor_theme** | MISSING | 光标主题配置 |
| 37 | **default_floating_border** | MISSING | 浮动窗口默认边框样式 |
| 38 | **swap** | MISSING | 容器交换 |
| 39 | **rename workspace** | MISSING | 工作区重命名 |
| 40 | **layout toggle** | PARTIAL | toggle 当前映射到 SplitH，应该循环切换布局 |

### P2 — 增强

| # | 功能 | 当前状态 | 差距描述 |
|---|------|----------|----------|
| 41 | bindswitch / bindgesture | MISSING | 笔记本开合/手势绑定 |
| 42 | bindsym 高级标志 | MISSING | --release, --locked, --whole-window 等 |
| 43 | opacity | MISSING | 窗口透明度 |
| 44 | sticky | MISSING | 浮动窗口跟随工作区切换 |
| 45 | inhibit_idle | MISSING | 空闲抑制 |
| 46 | title_format / titlebar_* | MISSING | 标题栏定制 |
| 47 | smart_gaps | MISSING | 智能间距 |
| 48 | per-edge outer gaps | MISSING | 各边独立外间距 |
| 49 | tray 支持 | MISSING | 系统托盘 |
| 50 | SEND_TICK / SYNC | MISSING | IPC tick 和 sync |
| 51 | no_focus | MISSING | 阻止特定窗口自动聚焦 |
| 52 | urgent | MISSING | 紧急状态管理 |
| 53 | max_render_time / allow_tearing | MISSING | 渲染优化 |
| 54 | 输入映射 (map_to_output/region) | MISSING | 绘图板映射 |
| 55 | popup_during_fullscreen | MISSING | 全屏弹窗处理 |

---

## 附录 A: rway 已实现功能汇总

### 配置解析 (rway-config)
- `set` 变量定义和替换
- `bindsym` 基本按键绑定（Mod1/Mod4/Shift/Control/Alt + 按键名）
- `exec` / `exec_always`
- `output` 配置（resolution, position, scale, transform）
- `input` 配置（通用 key-value 解析，不与 libinput 集成）
- `gaps inner/outer`
- `default_border normal|pixel|none`
- `for_window` 窗口规则（floating enable/disable, move to workspace, resize）
- `workspace <name> output <outputs>` 绑定
- `font`
- `focus_follows_mouse yes|no`

### 平铺引擎 (rway-tiling)
- Arena-allocated N 叉树
- SplitH/SplitV 比例分配布局
- Tabbed/Stacked 布局（仅聚焦子节点可见）
- 窗口插入/移除/空容器清理
- 方向焦点移动（left/right/up/down）
- 布局切换（SplitH/SplitV/Tabbed/Stacked）
- 工作区创建/切换/列表
- 浮动状态切换
- Inner/outer gaps 计算

### IPC 协议 (rway-ipc)
- i3-ipc 二进制协议编解码
- 所有消息类型和事件类型枚举定义
- GET_WORKSPACES, GET_OUTPUTS, GET_TREE, GET_VERSION 响应生成
- GET_INPUTS, GET_SEATS 存根（空列表）
- RUN_COMMAND 存根（固定 success）
- Unix socket 服务端（非阻塞，calloop 定时器轮询）
- 事件订阅解析（parse_subscribe_payload）
- WorkspaceEvent / WindowEvent 序列化

### 合成器核心 (rway)
- Smithay 集成（compositor, xdg_shell, shm, output, seat, layer_shell, xdg_decoration）
- Winit 和 udev 后端
- XWayland 支持（可选 feature）
- 按键绑定匹配和动作执行（exec, focus, workspace, split, layout, kill, toggle_floating, fullscreen, reload, exit）
- 窗口动画（ease-out cubic easing）
- 边框渲染
- Move grab / Resize grab（浮动窗口鼠标拖拽）
- Layer shell 支持

## 附录 B: IPC 协议类型编码参考

```
// 消息类型 (client -> server)
RUN_COMMAND     = 0
GET_WORKSPACES  = 1
SUBSCRIBE       = 2
GET_OUTPUTS     = 3
GET_TREE        = 4
GET_MARKS       = 5
GET_BAR_CONFIG  = 6
GET_VERSION     = 7
GET_BINDING_MODES = 8
GET_CONFIG      = 9
SEND_TICK       = 10
SYNC            = 11
GET_BINDING_STATE = 12
GET_INPUTS      = 100
GET_SEATS       = 101

// 事件类型 (server -> client, 最高位 = 1)
WORKSPACE       = 0x80000000
OUTPUT          = 0x80000001
MODE            = 0x80000002
WINDOW          = 0x80000003
BARCONFIG_UPDATE = 0x80000004
BINDING         = 0x80000005
SHUTDOWN        = 0x80000006
TICK            = 0x80000007
BAR_STATE_UPDATE = 0x80000014
INPUT           = 0x80000015
```

## 附录 C: 配置解析器支持的指令关键字

当前 `parse_directive()` 支持的顶级关键字：
```
set, bindsym, output, input, exec_always, exec, gaps, default_border,
for_window, workspace, font, focus_follows_mouse
```

未支持（静默忽略）的常见顶级关键字：
```
include, bar, default_orientation, workspace_layout, xwayland,
swaybg_command, swaynag_command, floating_modifier, floating_maximum_size,
floating_minimum_size, focus_on_window_activation, focus_wrapping,
mouse_warping, popup_during_fullscreen, primary_selection, show_marks,
tiling_drag, tiling_drag_threshold, title_align,
workspace_auto_back_and_forth, mode, assign, no_focus,
client.focused, client.focused_inactive, client.unfocused, client.urgent,
client.background, client.placeholder, default_floating_border,
hide_edge_borders, smart_borders, smart_gaps, force_display_urgency_hint,
titlebar_border_thickness, titlebar_padding, bindcode, bindswitch,
bindgesture, unbindsym, unbindcode, unbindswitch, unbindgesture, seat
```

---

## 实现指引

> 以下为所有 P0 优先级的 `[MISSING]` 和 `[PARTIAL]` 项的实现指引。
> 每项包含 Sway 精确行为、需要修改的文件、对应的 harness 测试位置。

### resize 命令 [P0] [MISSING]

**Sway 行为**: `resize shrink|grow width|height [amount [px|ppt]]`
- 默认增量：平铺窗口 10 ppt，浮动窗口 10 px
- `ppt`（percentage points）：调整容器在其父级中的比例（修改 `sizes[]` 数组）
- `px`（pixels）：调整像素大小（浮动窗口直接改 geometry）
- 平铺窗口 resize 影响相邻兄弟的 `sizes[]`（比例重新分配）

**需要修改的文件**:
1. `rway-config/src/types.rs` — 新增 `Action::Resize { grow: bool, axis: ResizeAxis, amount: i32, unit: ResizeUnit }` 变体，以及 `ResizeAxis`（Width/Height）、`ResizeUnit`（Px/Ppt）枚举
2. `rway-config/src/parser.rs` — 在 `parse_action()` 中解析 `resize shrink|grow width|height [amount [px|ppt]]`
3. `rway-tiling/src/commands.rs` — 新增 `resize_container(tree, node_id, axis, delta_ppt)` 函数，修改目标节点和相邻兄弟的 `sizes[]`
4. `rway/src/input/keybindings.rs` — 在 `execute_action()` 中添加 `Action::Resize` 分支，区分平铺/浮动窗口

**对应 harness 测试**: `rway-harness/src/categories/window.rs`

---

### move 方向命令 [P0] [MISSING]

**Sway 行为**: `move left|right|up|down [<px> px]`
- 平铺窗口：在 SplitH/SplitV 容器内与相邻兄弟交换位置
- 在容器边界移动时：将窗口提升到父容器的相邻位置
- 跨输出边界：窗口转移到相邻输出的活跃工作区
- 浮动窗口：按像素移动（默认 10px）

**需要修改的文件**:
1. `rway-tiling/src/commands.rs` — 新增 `move_window(tree, direction) -> bool` 函数：
   - 找到当前聚焦窗口
   - 若方向与父容器布局匹配（SplitH + Left/Right, SplitV + Up/Down）→ 交换 `children[]` 和 `sizes[]` 中的位置
   - 若方向与布局不匹配 → 向上遍历找到匹配的祖先容器
   - 边界处理：从当前容器移出到父容器
2. `rway/src/input/keybindings.rs` — 将 `Action::Move` 分支的 TODO 替换为调用 `tiling::commands::move_window()` + `state.relayout()`

**对应 harness 测试**: `rway-harness/src/categories/tiling.rs`

---

### move to workspace [P0] [MISSING]

**Sway 行为**: `move [container|window] to workspace <name>`
- 从当前工作区的平铺树中移除窗口
- 添加到目标工作区（若不存在则创建）
- 焦点保持在原工作区

**需要修改的文件**:
1. `rway-tiling/src/commands.rs` — 新增 `move_to_workspace(tree, window_id, target_ws_name) -> bool` 函数：
   - `remove_window()` 从当前位置移除
   - `insert_window_into()` 插入到目标工作区
2. `rway-tiling/src/workspace.rs` — 可能需要新增 `find_or_create_workspace()` 辅助函数
3. `rway/src/input/keybindings.rs` — 将 `Action::MoveToWorkspace` 分支的 TODO 替换为实际调用

**对应 harness 测试**: `rway-harness/src/categories/workspace.rs`

---

### focus parent/child [P0] [MISSING]

**Sway 行为**: `focus parent` / `focus child`
- `focus parent`：将焦点从当前窗口/容器上移到其父容器
- `focus child`：将焦点从当前容器下移到其 `focused_child` 指向的子节点
- 可连续按 `focus parent` 多次，一直上移到工作区层级

**需要修改的文件**:
1. `rway-config/src/types.rs` — 扩展 `Action` 枚举，新增 `FocusParent` 和 `FocusChild` 变体
2. `rway-config/src/parser.rs` — 在 `parse_action()` 中解析 `focus parent` 和 `focus child`
3. `rway-tiling/src/commands.rs` — 新增 `focus_parent(tree) -> bool` 和 `focus_child(tree) -> bool` 函数：
   - 需要在 Tree 中维护一个"当前焦点层级"概念（可能需要新增 `focused_node: Option<NodeId>` 字段）
4. `rway/src/input/keybindings.rs` — 添加 `Action::FocusParent` / `Action::FocusChild` 分支

**对应 harness 测试**: `rway-harness/src/categories/window.rs`

---

### floating_modifier [P0] [MISSING]

**Sway 行为**: `floating_modifier <modifier> [normal|inverse]`
- `normal`（默认）：修饰键 + 左键 = 拖拽移动浮动窗口，修饰键 + 右键 = 拖拽调整大小
- `inverse`：左右键行为反转
- 修饰键通常为 `Mod4`（Super）

**需要修改的文件**:
1. `rway-config/src/types.rs` — 在 `Config` 中新增 `floating_modifier: Option<Modifier>` 和 `floating_modifier_inverse: bool` 字段
2. `rway-config/src/parser.rs` — 在 `parse_directive()` 中添加 `"floating_modifier"` 分支
3. `rway/src/input/mod.rs` — 在鼠标按键事件处理中：检查是否按住 floating_modifier → 判断左键/右键 → 启动 move grab 或 resize grab
4. `rway/src/grabs/` — 复用已有的 MoveSurfaceGrab / ResizeSurfaceGrab

**对应 harness 测试**: `rway-harness/src/categories/config.rs`

---

### IPC SUBSCRIBE [P0] [MISSING]

**Sway 行为**: 消息类型 2，客户端发送 `["workspace", "window", ...]` JSON 数组
- 服务端回复 `{"success": true}`
- 之后当对应事件发生时，服务端主动推送事件到该连接
- 连接保持打开直到客户端断开
- waybar 依赖此功能实时更新工作区状态

**需要修改的文件**:
1. `rway-ipc/src/server.rs` — 扩展 `IpcServer`：
   - 新增 `subscribers: Vec<(UnixStream, HashSet<SubscriptionType>)>` 字段
   - 新增 `subscribe()` 方法注册订阅
   - 新增 `broadcast_event()` 方法向匹配的订阅者推送事件
2. `rway/src/ipc.rs` — 在 `dispatch_ipc_message()` 中：
   - `type=2` 分支：解析订阅 payload（`parse_subscribe_payload` 已实现），注册到 IpcServer
   - 保持连接打开（当前处理完即关闭，需改为持久连接模型）
3. `rway/src/state.rs` 或 `rway/src/shell.rs` — 在窗口/工作区状态变更时调用 `ipc_server.broadcast_event()`

**对应 harness 测试**: `rway-harness/src/categories/ipc.rs`

**注意**: 这是最复杂的 P0 项，需要将 IPC 从「请求-响应」模式改为「持久连接 + 事件推送」模式。

---

### IPC RUN_COMMAND 实际执行 [P0] [PARTIAL]

**当前状态**: `rway/src/ipc.rs:103` 固定返回 `[{"success": true}]`，不解析不执行。

**Sway 行为**: 接收命令字符串（可用分号分隔多个命令），解析并执行，返回每个命令的执行结果。

**需要修改的文件**:
1. `rway/src/ipc.rs` — `handle_run_command()` 改为：
   - 按分号分割命令字符串
   - 对每个命令调用 `rway_config::parse()` 的单条命令解析（或新建命令解析函数）
   - 调用 `execute_action()` 执行
   - 收集每个命令的 `CommandResult`
2. `rway-config/src/parser.rs` — 可能需要新增 `parse_command(cmd: &str) -> Result<Action, ParseError>` 单条命令解析函数
3. `rway/src/input/keybindings.rs` — `execute_action()` 可能需要提取为独立模块，使 IPC 和按键都可调用

**对应 harness 测试**: `rway-harness/src/categories/ipc.rs`

---

### fullscreen enable/disable [P0] [PARTIAL]

**当前状态**: 只有 `toggle_fullscreen()`，无法显式 `enable` 或 `disable`。

**Sway 行为**: `fullscreen [enable|disable|toggle] [global]`
- `enable`：强制进入全屏（已全屏则无操作）
- `disable`：强制退出全屏（未全屏则无操作）
- `toggle`：切换
- `global`：跨所有输出全屏（`fullscreen_mode=2`）

**需要修改的文件**:
1. `rway-config/src/types.rs` — 将 `Action::Fullscreen` 扩展为 `Action::Fullscreen(FullscreenAction)` 其中 `FullscreenAction` 枚举包含 `Enable, Disable, Toggle`
2. `rway-config/src/parser.rs` — 解析 `fullscreen enable|disable|toggle`
3. `rway-tiling/src/commands.rs` — 新增 `set_fullscreen(tree, window_id, enable: bool)` 和现有 `toggle_fullscreen()` 改为调用它
4. `rway/src/input/keybindings.rs` — 更新 `Action::Fullscreen` 匹配分支

**对应 harness 测试**: `rway-harness/src/categories/window.rs`

---

### floating enable/disable [P0] [PARTIAL]

**当前状态**: 只有 `toggle_floating()`，无法显式 `enable` 或 `disable`。

**Sway 行为**: `floating enable|disable|toggle`
- `enable`：强制浮动（已浮动则无操作）
- `disable`：强制平铺（已平铺则无操作）
- `toggle`：切换
- 浮动窗口从平铺树移到 `floating_nodes` 列表，居中显示

**需要修改的文件**:
1. `rway-config/src/types.rs` — 将 `Action::ToggleFloating` 扩展为 `Action::Floating(FloatingAction)` 其中 `FloatingAction` 枚举包含 `Enable, Disable, Toggle`
2. `rway-config/src/parser.rs` — 解析 `floating enable|disable|toggle`
3. `rway-tiling/src/commands.rs` — 新增 `set_floating(tree, window_id, enable: bool)` 和现有 `toggle_floating()` 改为调用它
4. `rway/src/input/keybindings.rs` — 更新匹配分支

**对应 harness 测试**: `rway-harness/src/categories/window.rs`

---

### xkb_layout / xkb_options [P0] [MISSING]

**Sway 行为**: `input <identifier> xkb_layout <layout>` / `input <identifier> xkb_options <options>`
- 设置键盘的 XKB 布局（如 `us`, `de`, `us,ru`）
- 设置 XKB 选项（如 `caps:escape`, `grp:alt_shift_toggle`）
- `<identifier>` 可以是具体设备名或 `type:keyboard` 通配符

**当前状态**: `rway-config/src/parser.rs` 已能解析 `input <id> <key> <value>` 为通用 key-value，`InputConfig.settings` 是 `HashMap<String, String>`。但 `rway/src/state.rs` 中键盘初始化硬编码为 `us` + `dvorak`。

**需要修改的文件**:
1. `rway-config/src/types.rs` — 可选：为 `InputConfig` 添加类型化的 `xkb_layout`, `xkb_options` 等字段（或保持 HashMap 并在使用处提取）
2. `rway/src/state.rs` — 在 `new_with_seat_name()` 中从 `config.inputs` 读取 `xkb_layout`、`xkb_variant`、`xkb_options`，替换硬编码的 `XkbConfig`
3. `rway/src/input/mod.rs` — 如需支持运行时切换布局，需添加 `seat.add_keyboard()` 的重新配置逻辑

**对应 harness 测试**: `rway-harness/src/categories/input.rs`

---

### focus_follows_mouse always 模式 [P0] [PARTIAL]

**当前状态**: `rway-config` 解析 `focus_follows_mouse yes|no` 为 `bool`，不支持 `always` 三态。

**Sway 行为**: `focus_follows_mouse yes|no|always`
- `yes`：鼠标进入窗口时聚焦，但新窗口创建时保留新窗口焦点
- `no`：不跟随鼠标
- `always`：始终跟随鼠标，即使有新窗口创建

**需要修改的文件**:
1. `rway-config/src/types.rs` — 将 `Config.focus_follows_mouse: bool` 改为 `FocusFollowsMouse` 枚举（Yes/No/Always）
2. `rway-config/src/parser.rs` — 解析三态值
3. `rway/src/focus.rs`（或相关鼠标事件处理文件）— 在焦点逻辑中区分 `yes` 和 `always`

**对应 harness 测试**: `rway-harness/src/categories/config.rs`
