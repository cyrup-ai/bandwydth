// Auto-generated spinner data - DO NOT EDIT
// Generated from assets/spinners.json

#![allow(clippy::unreadable_literal)]

use core::time::Duration;

/// Compile-time spinner metadata (interval + frame table)
#[derive(Copy, Clone, Debug)]
pub struct SpinnerFrames {
    /// Frame update period
    pub interval: Duration,
    /// Ordered frames to show
    pub frames: &'static [&'static str],
}

// Helper - const-friendly millisecond constructor
const fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

// Default spinners always available
const DEFAULT_DOTS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DEFAULT_LINE: &[&str] = &["-", "\\", "|", "/"];
const DEFAULT_ARC: &[&str] = &["◜", "◠", "◝", "◞", "◡", "◟"];
const DEFAULT_BOUNCING_BAR: &[&str] = &[
    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]", "[    ]",
    "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
];
const DEFAULT_CIRCLE_QUARTERS: &[&str] = &["◴", "◷", "◶", "◵"];
const DEFAULT_TOGGLE: &[&str] = &["⊶", "⊷"];

#[cfg(feature = "aesthetic")]
const AESTHETIC: &[&str] = &[
    "▰▱▱▱▱▱▱",
    "▰▰▱▱▱▱▱",
    "▰▰▰▱▱▱▱",
    "▰▰▰▰▱▱▱",
    "▰▰▰▰▰▱▱",
    "▰▰▰▰▰▰▱",
    "▰▰▰▰▰▰▰",
    "▰▱▱▱▱▱▱",
];

#[cfg(feature = "arrow")]
const ARROW: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];

#[cfg(feature = "arrow2")]
const ARROW2: &[&str] = &["⬆️ ", "↗️ ", "➡️ ", "↘️ ", "⬇️ ", "↙️ ", "⬅️ ", "↖️ "];

#[cfg(feature = "arrow3")]
const ARROW3: &[&str] = &["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"];

#[cfg(feature = "balloon")]
const BALLOON: &[&str] = &[" ", ".", "o", "O", "@", "*", " "];

#[cfg(feature = "balloon2")]
const BALLOON2: &[&str] = &[".", "o", "O", "°", "O", "o", "."];

#[cfg(feature = "beta_wave")]
const BETA_WAVE: &[&str] = &[
    "ρββββββ",
    "βρβββββ",
    "ββρββββ",
    "βββρβββ",
    "ββββρββ",
    "βββββρβ",
    "ββββββρ",
];

#[cfg(feature = "binary")]
const BINARY: &[&str] = &[
    "010010", "001100", "100101", "111010", "111101", "010111", "101011", "111000", "110011",
    "110101",
];

#[cfg(feature = "blue_pulse")]
const BLUE_PULSE: &[&str] = &["🔹 ", "🔷 ", "🔵 ", "🔵 ", "🔷 "];

#[cfg(feature = "bounce")]
const BOUNCE: &[&str] = &["⠁", "⠂", "⠄", "⠂"];

#[cfg(feature = "bouncing_ball")]
const BOUNCING_BALL: &[&str] = &[
    "( ●    )",
    "(  ●   )",
    "(   ●  )",
    "(    ● )",
    "(     ●)",
    "(    ● )",
    "(   ●  )",
    "(  ●   )",
    "( ●    )",
    "(●     )",
];

#[cfg(feature = "box_bounce")]
const BOX_BOUNCE: &[&str] = &["▖", "▘", "▝", "▗"];

#[cfg(feature = "box_bounce2")]
const BOX_BOUNCE2: &[&str] = &["▌", "▀", "▐", "▄"];

#[cfg(feature = "christmas")]
const CHRISTMAS: &[&str] = &["🌲", "🎄"];

#[cfg(feature = "circle")]
const CIRCLE: &[&str] = &["◡", "⊙", "◠"];

#[cfg(feature = "circle_halves")]
const CIRCLE_HALVES: &[&str] = &["◐", "◓", "◑", "◒"];

#[cfg(feature = "clock")]
const CLOCK: &[&str] = &[
    "🕛 ", "🕐 ", "🕑 ", "🕒 ", "🕓 ", "🕔 ", "🕕 ", "🕖 ", "🕗 ", "🕘 ", "🕙 ", "🕚 ",
];

#[cfg(feature = "dots10")]
const DOTS10: &[&str] = &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"];

#[cfg(feature = "dots11")]
const DOTS11: &[&str] = &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"];

#[cfg(feature = "dots12")]
const DOTS12: &[&str] = &[
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀",
];

#[cfg(feature = "dots13")]
const DOTS13: &[&str] = &["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶"];

#[cfg(feature = "dots14")]
const DOTS14: &[&str] = &[
    "⠉⠉", "⠈⠙", "⠀⠹", "⠀⢸", "⠀⣰", "⢀⣠", "⣀⣀", "⣄⡀", "⣆⠀", "⡇⠀", "⠏⠀", "⠋⠁",
];

#[cfg(feature = "dots2")]
const DOTS2: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

#[cfg(feature = "dots3")]
const DOTS3: &[&str] = &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"];

#[cfg(feature = "dots4")]
const DOTS4: &[&str] = &[
    "⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠰", "⠸", "⠙", "⠋", "⠇", "⠆",
];

#[cfg(feature = "dots5")]
const DOTS5: &[&str] = &[
    "⠋", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋",
];

#[cfg(feature = "dots6")]
const DOTS6: &[&str] = &[
    "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠴", "⠲", "⠒", "⠂", "⠂",
    "⠒", "⠚", "⠙", "⠉", "⠁",
];

#[cfg(feature = "dots7")]
const DOTS7: &[&str] = &[
    "⠈", "⠉", "⠋", "⠓", "⠒", "⠐", "⠐", "⠒", "⠖", "⠦", "⠤", "⠠", "⠠", "⠤", "⠦", "⠖", "⠒", "⠐", "⠐",
    "⠒", "⠓", "⠋", "⠉", "⠈",
];

#[cfg(feature = "dots8")]
const DOTS8: &[&str] = &[
    "⠁", "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠠", "⠠", "⠤", "⠦",
    "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈", "⠈",
];

#[cfg(feature = "dots8_bit")]
const DOTS8_BIT: &[&str] = &[
    "⠀", "⠁", "⠂", "⠃", "⠄", "⠅", "⠆", "⠇", "⡀", "⡁", "⡂", "⡃", "⡄", "⡅", "⡆", "⡇", "⠈", "⠉", "⠊",
    "⠋", "⠌", "⠍", "⠎", "⠏", "⡈", "⡉", "⡊", "⡋", "⡌", "⡍", "⡎", "⡏", "⠐", "⠑", "⠒", "⠓", "⠔", "⠕",
    "⠖", "⠗", "⡐", "⡑", "⡒", "⡓", "⡔", "⡕", "⡖", "⡗", "⠘", "⠙", "⠚", "⠛", "⠜", "⠝", "⠞", "⠟", "⡘",
    "⡙", "⡚", "⡛", "⡜", "⡝", "⡞", "⡟", "⠠", "⠡", "⠢", "⠣", "⠤", "⠥", "⠦", "⠧", "⡠", "⡡", "⡢", "⡣",
    "⡤", "⡥", "⡦", "⡧", "⠨", "⠩", "⠪", "⠫", "⠬", "⠭", "⠮", "⠯", "⡨", "⡩", "⡪", "⡫", "⡬", "⡭", "⡮",
    "⡯", "⠰", "⠱", "⠲", "⠳", "⠴", "⠵", "⠶", "⠷", "⡰", "⡱", "⡲", "⡳", "⡴", "⡵", "⡶", "⡷", "⠸", "⠹",
    "⠺", "⠻", "⠼", "⠽", "⠾", "⠿", "⡸", "⡹", "⡺", "⡻", "⡼", "⡽", "⡾", "⡿", "⢀", "⢁", "⢂", "⢃", "⢄",
    "⢅", "⢆", "⢇", "⣀", "⣁", "⣂", "⣃", "⣄", "⣅", "⣆", "⣇", "⢈", "⢉", "⢊", "⢋", "⢌", "⢍", "⢎", "⢏",
    "⣈", "⣉", "⣊", "⣋", "⣌", "⣍", "⣎", "⣏", "⢐", "⢑", "⢒", "⢓", "⢔", "⢕", "⢖", "⢗", "⣐", "⣑", "⣒",
    "⣓", "⣔", "⣕", "⣖", "⣗", "⢘", "⢙", "⢚", "⢛", "⢜", "⢝", "⢞", "⢟", "⣘", "⣙", "⣚", "⣛", "⣜", "⣝",
    "⣞", "⣟", "⢠", "⢡", "⢢", "⢣", "⢤", "⢥", "⢦", "⢧", "⣠", "⣡", "⣢", "⣣", "⣤", "⣥", "⣦", "⣧", "⢨",
    "⢩", "⢪", "⢫", "⢬", "⢭", "⢮", "⢯", "⣨", "⣩", "⣪", "⣫", "⣬", "⣭", "⣮", "⣯", "⢰", "⢱", "⢲", "⢳",
    "⢴", "⢵", "⢶", "⢷", "⣰", "⣱", "⣲", "⣳", "⣴", "⣵", "⣶", "⣷", "⢸", "⢹", "⢺", "⢻", "⢼", "⢽", "⢾",
    "⢿", "⣸", "⣹", "⣺", "⣻", "⣼", "⣽", "⣾", "⣿",
];

#[cfg(feature = "dots9")]
const DOTS9: &[&str] = &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"];

#[cfg(feature = "dots_circle")]
const DOTS_CIRCLE: &[&str] = &["⢎ ", "⠎⠁", "⠊⠑", "⠈⠱", " ⡱", "⢀⡰", "⢄⡠", "⢆⡀"];

#[cfg(feature = "dqpb")]
const DQPB: &[&str] = &["d", "q", "p", "b"];

#[cfg(feature = "dwarf_fortress")]
const DWARF_FORTRESS: &[&str] = &[
    " ██████£££  ",
    "☺██████£££  ",
    "☺██████£££  ",
    "☺▓█████£££  ",
    "☺▓█████£££  ",
    "☺▒█████£££  ",
    "☺▒█████£££  ",
    "☺░█████£££  ",
    "☺░█████£££  ",
    "☺ █████£££  ",
    " ☺█████£££  ",
    " ☺█████£££  ",
    " ☺▓████£££  ",
    " ☺▓████£££  ",
    " ☺▒████£££  ",
    " ☺▒████£££  ",
    " ☺░████£££  ",
    " ☺░████£££  ",
    " ☺ ████£££  ",
    "  ☺████£££  ",
    "  ☺████£££  ",
    "  ☺▓███£££  ",
    "  ☺▓███£££  ",
    "  ☺▒███£££  ",
    "  ☺▒███£££  ",
    "  ☺░███£££  ",
    "  ☺░███£££  ",
    "  ☺ ███£££  ",
    "   ☺███£££  ",
    "   ☺███£££  ",
    "   ☺▓██£££  ",
    "   ☺▓██£££  ",
    "   ☺▒██£££  ",
    "   ☺▒██£££  ",
    "   ☺░██£££  ",
    "   ☺░██£££  ",
    "   ☺ ██£££  ",
    "    ☺██£££  ",
    "    ☺██£££  ",
    "    ☺▓█£££  ",
    "    ☺▓█£££  ",
    "    ☺▒█£££  ",
    "    ☺▒█£££  ",
    "    ☺░█£££  ",
    "    ☺░█£££  ",
    "    ☺ █£££  ",
    "     ☺█£££  ",
    "     ☺█£££  ",
    "     ☺▓£££  ",
    "     ☺▓£££  ",
    "     ☺▒£££  ",
    "     ☺▒£££  ",
    "     ☺░£££  ",
    "     ☺░£££  ",
    "     ☺ £££  ",
    "      ☺£££  ",
    "      ☺£££  ",
    "      ☺▓££  ",
    "      ☺▓££  ",
    "      ☺▒££  ",
    "      ☺▒££  ",
    "      ☺░££  ",
    "      ☺░££  ",
    "      ☺ ££  ",
    "       ☺££  ",
    "       ☺££  ",
    "       ☺▓£  ",
    "       ☺▓£  ",
    "       ☺▒£  ",
    "       ☺▒£  ",
    "       ☺░£  ",
    "       ☺░£  ",
    "       ☺ £  ",
    "        ☺£  ",
    "        ☺£  ",
    "        ☺▓  ",
    "        ☺▓  ",
    "        ☺▒  ",
    "        ☺▒  ",
    "        ☺░  ",
    "        ☺░  ",
    "        ☺   ",
    "        ☺  &",
    "        ☺ ☼&",
    "       ☺ ☼ &",
    "       ☺☼  &",
    "      ☺☼  & ",
    "      ‼   & ",
    "     ☺   &  ",
    "    ‼    &  ",
    "   ☺    &   ",
    "  ‼     &   ",
    " ☺     &    ",
    "‼      &    ",
    "      &     ",
    "      &     ",
    "     &   ░  ",
    "     &   ▒  ",
    "    &    ▓  ",
    "    &    £  ",
    "   &    ░£  ",
    "   &    ▒£  ",
    "  &     ▓£  ",
    "  &     ££  ",
    " &     ░££  ",
    " &     ▒££  ",
    "&      ▓££  ",
    "&      £££  ",
    "      ░£££  ",
    "      ▒£££  ",
    "      ▓£££  ",
    "      █£££  ",
    "     ░█£££  ",
    "     ▒█£££  ",
    "     ▓█£££  ",
    "     ██£££  ",
    "    ░██£££  ",
    "    ▒██£££  ",
    "    ▓██£££  ",
    "    ███£££  ",
    "   ░███£££  ",
    "   ▒███£££  ",
    "   ▓███£££  ",
    "   ████£££  ",
    "  ░████£££  ",
    "  ▒████£££  ",
    "  ▓████£££  ",
    "  █████£££  ",
    " ░█████£££  ",
    " ▒█████£££  ",
    " ▓█████£££  ",
    " ██████£££  ",
    " ██████£££  ",
];

#[cfg(feature = "earth")]
const EARTH: &[&str] = &["🌍 ", "🌎 ", "🌏 "];

#[cfg(feature = "finger_dance")]
const FINGER_DANCE: &[&str] = &["🤘 ", "🤟 ", "🖖 ", "✋ ", "🤚 ", "👆 "];

#[cfg(feature = "fist_bump")]
const FIST_BUMP: &[&str] = &[
    "🤜　　　　🤛 ",
    "🤜　　　　🤛 ",
    "🤜　　　　🤛 ",
    "　🤜　　🤛　 ",
    "　　🤜🤛　　 ",
    "　🤜✨🤛　　 ",
    "🤜　✨　🤛　 ",
];

#[cfg(feature = "flip")]
const FLIP: &[&str] = &["_", "_", "_", "-", "`", "`", "'", "´", "-", "_", "_", "_"];

#[cfg(feature = "grenade")]
const GRENADE: &[&str] = &[
    "،  ", "′  ", " ´ ", " ‾ ", "  ⸌", "  ⸊", "  |", "  ⁎", "  ⁕", " ෴ ", "  ⁓", "   ", "   ",
    "   ",
];

#[cfg(feature = "grow_horizontal")]
const GROW_HORIZONTAL: &[&str] = &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▊", "▋", "▌", "▍", "▎"];

#[cfg(feature = "grow_vertical")]
const GROW_VERTICAL: &[&str] = &["▁", "▃", "▄", "▅", "▆", "▇", "▆", "▅", "▄", "▃"];

#[cfg(feature = "hamburger")]
const HAMBURGER: &[&str] = &["☱", "☲", "☴"];

#[cfg(feature = "hearts")]
const HEARTS: &[&str] = &["💛 ", "💙 ", "💜 ", "💚 ", "❤️ "];

#[cfg(feature = "layer")]
const LAYER: &[&str] = &["-", "=", "≡"];

#[cfg(feature = "line2")]
const LINE2: &[&str] = &["⠂", "-", "–", "—", "–", "-"];

#[cfg(feature = "material")]
const MATERIAL: &[&str] = &[
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███████▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "██████████▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "█████████████▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁██████████████▁▁▁▁",
    "▁▁▁██████████████▁▁▁",
    "▁▁▁▁█████████████▁▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁██████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁▁█████████████▁▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁▁███████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁▁█████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
];

#[cfg(feature = "mindblown")]
const MINDBLOWN: &[&str] = &[
    "😐 ", "😐 ", "😮 ", "😮 ", "😦 ", "😦 ", "😧 ", "😧 ", "🤯 ", "💥 ", "✨ ", "　 ", "　 ",
    "　 ",
];

#[cfg(feature = "monkey")]
const MONKEY: &[&str] = &["🙈 ", "🙈 ", "🙉 ", "🙊 "];

#[cfg(feature = "moon")]
const MOON: &[&str] = &["🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 "];

#[cfg(feature = "noise")]
const NOISE: &[&str] = &["▓", "▒", "░"];

#[cfg(feature = "orange_blue_pulse")]
const ORANGE_BLUE_PULSE: &[&str] = &[
    "🔸 ", "🔶 ", "🟠 ", "🟠 ", "🔶 ", "🔹 ", "🔷 ", "🔵 ", "🔵 ", "🔷 ",
];

#[cfg(feature = "orange_pulse")]
const ORANGE_PULSE: &[&str] = &["🔸 ", "🔶 ", "🟠 ", "🟠 ", "🔶 "];

#[cfg(feature = "pipe")]
const PIPE: &[&str] = &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"];

#[cfg(feature = "point")]
const POINT: &[&str] = &["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙"];

#[cfg(feature = "pong")]
const PONG: &[&str] = &[
    "▐⠂       ▌",
    "▐⠈       ▌",
    "▐ ⠂      ▌",
    "▐ ⠠      ▌",
    "▐  ⡀     ▌",
    "▐  ⠠     ▌",
    "▐   ⠂    ▌",
    "▐   ⠈    ▌",
    "▐    ⠂   ▌",
    "▐    ⠠   ▌",
    "▐     ⡀  ▌",
    "▐     ⠠  ▌",
    "▐      ⠂ ▌",
    "▐      ⠈ ▌",
    "▐       ⠂▌",
    "▐       ⠠▌",
    "▐       ⡀▌",
    "▐      ⠠ ▌",
    "▐      ⠂ ▌",
    "▐     ⠈  ▌",
    "▐     ⠂  ▌",
    "▐    ⠠   ▌",
    "▐    ⡀   ▌",
    "▐   ⠠    ▌",
    "▐   ⠂    ▌",
    "▐  ⠈     ▌",
    "▐  ⠂     ▌",
    "▐ ⠠      ▌",
    "▐ ⡀      ▌",
    "▐⠠       ▌",
];

#[cfg(feature = "runner")]
const RUNNER: &[&str] = &["🚶 ", "🏃 "];

#[cfg(feature = "sand")]
const SAND: &[&str] = &[
    "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦", "⣮", "⣶", "⣷",
    "⣿", "⡿", "⠿", "⢟", "⠟", "⡛", "⠛", "⠫", "⢋", "⠋", "⠍", "⡉", "⠉", "⠑", "⠡", "⢁",
];

#[cfg(feature = "shark")]
const SHARK: &[&str] = &[
    "▐|\\____________▌",
    "▐_|\\___________▌",
    "▐__|\\__________▌",
    "▐___|\\_________▌",
    "▐____|\\________▌",
    "▐_____|\\_______▌",
    "▐______|\\______▌",
    "▐_______|\\_____▌",
    "▐________|\\____▌",
    "▐_________|\\___▌",
    "▐__________|\\__▌",
    "▐___________|\\_▌",
    "▐____________|\\▌",
    "▐____________/|▌",
    "▐___________/|_▌",
    "▐__________/|__▌",
    "▐_________/|___▌",
    "▐________/|____▌",
    "▐_______/|_____▌",
    "▐______/|______▌",
    "▐_____/|_______▌",
    "▐____/|________▌",
    "▐___/|_________▌",
    "▐__/|__________▌",
    "▐_/|___________▌",
    "▐/|____________▌",
];

#[cfg(feature = "simple_dots")]
const SIMPLE_DOTS: &[&str] = &[".  ", ".. ", "...", "   "];

#[cfg(feature = "simple_dots_scrolling")]
const SIMPLE_DOTS_SCROLLING: &[&str] = &[".  ", ".. ", "...", " ..", "  .", "   "];

#[cfg(feature = "smiley")]
const SMILEY: &[&str] = &["😄 ", "😝 "];

#[cfg(feature = "soccer_header")]
const SOCCER_HEADER: &[&str] = &[
    " 🧑⚽️       🧑 ",
    "🧑  ⚽️      🧑 ",
    "🧑   ⚽️     🧑 ",
    "🧑    ⚽️    🧑 ",
    "🧑     ⚽️   🧑 ",
    "🧑      ⚽️  🧑 ",
    "🧑       ⚽️🧑  ",
    "🧑      ⚽️  🧑 ",
    "🧑     ⚽️   🧑 ",
    "🧑    ⚽️    🧑 ",
    "🧑   ⚽️     🧑 ",
    "🧑  ⚽️      🧑 ",
];

#[cfg(feature = "speaker")]
const SPEAKER: &[&str] = &["🔈 ", "🔉 ", "🔊 ", "🔉 "];

#[cfg(feature = "square_corners")]
const SQUARE_CORNERS: &[&str] = &["◰", "◳", "◲", "◱"];

#[cfg(feature = "squish")]
const SQUISH: &[&str] = &["╫", "╪"];

#[cfg(feature = "star")]
const STAR: &[&str] = &["✶", "✸", "✹", "✺", "✹", "✷"];

#[cfg(feature = "star2")]
const STAR2: &[&str] = &["+", "x", "*"];

#[cfg(feature = "time_travel")]
const TIME_TRAVEL: &[&str] = &[
    "🕛 ", "🕚 ", "🕙 ", "🕘 ", "🕗 ", "🕖 ", "🕕 ", "🕔 ", "🕓 ", "🕒 ", "🕑 ", "🕐 ",
];

#[cfg(feature = "toggle10")]
const TOGGLE10: &[&str] = &["㊂", "㊀", "㊁"];

#[cfg(feature = "toggle11")]
const TOGGLE11: &[&str] = &["⧇", "⧆"];

#[cfg(feature = "toggle12")]
const TOGGLE12: &[&str] = &["☗", "☖"];

#[cfg(feature = "toggle13")]
const TOGGLE13: &[&str] = &["=", "*", "-"];

#[cfg(feature = "toggle2")]
const TOGGLE2: &[&str] = &["▫", "▪"];

#[cfg(feature = "toggle3")]
const TOGGLE3: &[&str] = &["□", "■"];

#[cfg(feature = "toggle4")]
const TOGGLE4: &[&str] = &["■", "□", "▪", "▫"];

#[cfg(feature = "toggle5")]
const TOGGLE5: &[&str] = &["▮", "▯"];

#[cfg(feature = "toggle6")]
const TOGGLE6: &[&str] = &["ဝ", "၀"];

#[cfg(feature = "toggle7")]
const TOGGLE7: &[&str] = &["⦾", "⦿"];

#[cfg(feature = "toggle8")]
const TOGGLE8: &[&str] = &["◍", "◌"];

#[cfg(feature = "toggle9")]
const TOGGLE9: &[&str] = &["◉", "◎"];

#[cfg(feature = "triangle")]
const TRIANGLE: &[&str] = &["◢", "◣", "◤", "◥"];

#[cfg(feature = "weather")]
const WEATHER: &[&str] = &[
    "☀️ ", "☀️ ", "☀️ ", "🌤 ", "⛅️ ", "🌥 ", "☁️ ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "🌧 ", "🌨 ", "⛈ ", "🌨 ",
    "🌧 ", "🌨 ", "☁️ ", "🌥 ", "⛅️ ", "🌤 ", "☀️ ", "☀️ ",
];

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum SpinnerPreset {
    // Default variants always available
    Dots,
    Line,
    Arc,
    BouncingBar,
    CircleQuarters,
    Toggle,
    #[cfg(feature = "aesthetic")]
    Aesthetic,
    #[cfg(feature = "arrow")]
    Arrow,
    #[cfg(feature = "arrow2")]
    Arrow2,
    #[cfg(feature = "arrow3")]
    Arrow3,
    #[cfg(feature = "balloon")]
    Balloon,
    #[cfg(feature = "balloon2")]
    Balloon2,
    #[cfg(feature = "beta_wave")]
    BetaWave,
    #[cfg(feature = "binary")]
    Binary,
    #[cfg(feature = "blue_pulse")]
    BluePulse,
    #[cfg(feature = "bounce")]
    Bounce,
    #[cfg(feature = "bouncing_ball")]
    BouncingBall,

    #[cfg(feature = "box_bounce")]
    BoxBounce,
    #[cfg(feature = "box_bounce2")]
    BoxBounce2,
    #[cfg(feature = "christmas")]
    Christmas,
    #[cfg(feature = "circle")]
    Circle,
    #[cfg(feature = "circle_halves")]
    CircleHalves,

    #[cfg(feature = "clock")]
    Clock,

    #[cfg(feature = "dots10")]
    Dots10,
    #[cfg(feature = "dots11")]
    Dots11,
    #[cfg(feature = "dots12")]
    Dots12,
    #[cfg(feature = "dots13")]
    Dots13,
    #[cfg(feature = "dots14")]
    Dots14,
    #[cfg(feature = "dots2")]
    Dots2,
    #[cfg(feature = "dots3")]
    Dots3,
    #[cfg(feature = "dots4")]
    Dots4,
    #[cfg(feature = "dots5")]
    Dots5,
    #[cfg(feature = "dots6")]
    Dots6,
    #[cfg(feature = "dots7")]
    Dots7,
    #[cfg(feature = "dots8")]
    Dots8,
    #[cfg(feature = "dots8_bit")]
    Dots8Bit,
    #[cfg(feature = "dots9")]
    Dots9,
    #[cfg(feature = "dots_circle")]
    DotsCircle,
    #[cfg(feature = "dqpb")]
    Dqpb,
    #[cfg(feature = "dwarf_fortress")]
    DwarfFortress,
    #[cfg(feature = "earth")]
    Earth,
    #[cfg(feature = "finger_dance")]
    FingerDance,
    #[cfg(feature = "fist_bump")]
    FistBump,
    #[cfg(feature = "flip")]
    Flip,
    #[cfg(feature = "grenade")]
    Grenade,
    #[cfg(feature = "grow_horizontal")]
    GrowHorizontal,
    #[cfg(feature = "grow_vertical")]
    GrowVertical,
    #[cfg(feature = "hamburger")]
    Hamburger,
    #[cfg(feature = "hearts")]
    Hearts,
    #[cfg(feature = "layer")]
    Layer,

    #[cfg(feature = "line2")]
    Line2,
    #[cfg(feature = "material")]
    Material,
    #[cfg(feature = "mindblown")]
    Mindblown,
    #[cfg(feature = "monkey")]
    Monkey,
    #[cfg(feature = "moon")]
    Moon,
    #[cfg(feature = "noise")]
    Noise,
    #[cfg(feature = "orange_blue_pulse")]
    OrangeBluePulse,
    #[cfg(feature = "orange_pulse")]
    OrangePulse,
    #[cfg(feature = "pipe")]
    Pipe,
    #[cfg(feature = "point")]
    Point,
    #[cfg(feature = "pong")]
    Pong,
    #[cfg(feature = "runner")]
    Runner,
    #[cfg(feature = "sand")]
    Sand,
    #[cfg(feature = "shark")]
    Shark,
    #[cfg(feature = "simple_dots")]
    SimpleDots,
    #[cfg(feature = "simple_dots_scrolling")]
    SimpleDotsScrolling,
    #[cfg(feature = "smiley")]
    Smiley,
    #[cfg(feature = "soccer_header")]
    SoccerHeader,
    #[cfg(feature = "speaker")]
    Speaker,
    #[cfg(feature = "square_corners")]
    SquareCorners,
    #[cfg(feature = "squish")]
    Squish,
    #[cfg(feature = "star")]
    Star,
    #[cfg(feature = "star2")]
    Star2,
    #[cfg(feature = "time_travel")]
    TimeTravel,

    #[cfg(feature = "toggle10")]
    Toggle10,
    #[cfg(feature = "toggle11")]
    Toggle11,
    #[cfg(feature = "toggle12")]
    Toggle12,
    #[cfg(feature = "toggle13")]
    Toggle13,
    #[cfg(feature = "toggle2")]
    Toggle2,
    #[cfg(feature = "toggle3")]
    Toggle3,
    #[cfg(feature = "toggle4")]
    Toggle4,
    #[cfg(feature = "toggle5")]
    Toggle5,
    #[cfg(feature = "toggle6")]
    Toggle6,
    #[cfg(feature = "toggle7")]
    Toggle7,
    #[cfg(feature = "toggle8")]
    Toggle8,
    #[cfg(feature = "toggle9")]
    Toggle9,
    #[cfg(feature = "triangle")]
    Triangle,
    #[cfg(feature = "weather")]
    Weather,
}

impl SpinnerPreset {
    /// Return the static frame data for this preset
    #[inline]
    pub const fn data(self) -> SpinnerFrames {
        match self {
            // Default variants
            SpinnerPreset::Dots => SpinnerFrames {
                interval: ms(80),
                frames: DEFAULT_DOTS,
            },
            SpinnerPreset::Line => SpinnerFrames {
                interval: ms(130),
                frames: DEFAULT_LINE,
            },
            SpinnerPreset::Arc => SpinnerFrames {
                interval: ms(100),
                frames: DEFAULT_ARC,
            },
            SpinnerPreset::BouncingBar => SpinnerFrames {
                interval: ms(80),
                frames: DEFAULT_BOUNCING_BAR,
            },
            SpinnerPreset::CircleQuarters => SpinnerFrames {
                interval: ms(120),
                frames: DEFAULT_CIRCLE_QUARTERS,
            },
            SpinnerPreset::Toggle => SpinnerFrames {
                interval: ms(250),
                frames: DEFAULT_TOGGLE,
            },
            #[cfg(feature = "aesthetic")]
            SpinnerPreset::Aesthetic => SpinnerFrames {
                interval: ms(80),
                frames: AESTHETIC,
            },

            #[cfg(feature = "arrow")]
            SpinnerPreset::Arrow => SpinnerFrames {
                interval: ms(100),
                frames: ARROW,
            },
            #[cfg(feature = "arrow2")]
            SpinnerPreset::Arrow2 => SpinnerFrames {
                interval: ms(80),
                frames: ARROW2,
            },
            #[cfg(feature = "arrow3")]
            SpinnerPreset::Arrow3 => SpinnerFrames {
                interval: ms(120),
                frames: ARROW3,
            },
            #[cfg(feature = "balloon")]
            SpinnerPreset::Balloon => SpinnerFrames {
                interval: ms(140),
                frames: BALLOON,
            },
            #[cfg(feature = "balloon2")]
            SpinnerPreset::Balloon2 => SpinnerFrames {
                interval: ms(120),
                frames: BALLOON2,
            },
            #[cfg(feature = "beta_wave")]
            SpinnerPreset::BetaWave => SpinnerFrames {
                interval: ms(80),
                frames: BETA_WAVE,
            },
            #[cfg(feature = "binary")]
            SpinnerPreset::Binary => SpinnerFrames {
                interval: ms(80),
                frames: BINARY,
            },
            #[cfg(feature = "blue_pulse")]
            SpinnerPreset::BluePulse => SpinnerFrames {
                interval: ms(100),
                frames: BLUE_PULSE,
            },
            #[cfg(feature = "bounce")]
            SpinnerPreset::Bounce => SpinnerFrames {
                interval: ms(120),
                frames: BOUNCE,
            },
            #[cfg(feature = "bouncing_ball")]
            SpinnerPreset::BouncingBall => SpinnerFrames {
                interval: ms(80),
                frames: BOUNCING_BALL,
            },

            #[cfg(feature = "box_bounce")]
            SpinnerPreset::BoxBounce => SpinnerFrames {
                interval: ms(120),
                frames: BOX_BOUNCE,
            },
            #[cfg(feature = "box_bounce2")]
            SpinnerPreset::BoxBounce2 => SpinnerFrames {
                interval: ms(100),
                frames: BOX_BOUNCE2,
            },
            #[cfg(feature = "christmas")]
            SpinnerPreset::Christmas => SpinnerFrames {
                interval: ms(400),
                frames: CHRISTMAS,
            },
            #[cfg(feature = "circle")]
            SpinnerPreset::Circle => SpinnerFrames {
                interval: ms(120),
                frames: CIRCLE,
            },
            #[cfg(feature = "circle_halves")]
            SpinnerPreset::CircleHalves => SpinnerFrames {
                interval: ms(50),
                frames: CIRCLE_HALVES,
            },

            #[cfg(feature = "clock")]
            SpinnerPreset::Clock => SpinnerFrames {
                interval: ms(100),
                frames: CLOCK,
            },

            #[cfg(feature = "dots10")]
            SpinnerPreset::Dots10 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS10,
            },
            #[cfg(feature = "dots11")]
            SpinnerPreset::Dots11 => SpinnerFrames {
                interval: ms(100),
                frames: DOTS11,
            },
            #[cfg(feature = "dots12")]
            SpinnerPreset::Dots12 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS12,
            },
            #[cfg(feature = "dots13")]
            SpinnerPreset::Dots13 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS13,
            },
            #[cfg(feature = "dots14")]
            SpinnerPreset::Dots14 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS14,
            },
            #[cfg(feature = "dots2")]
            SpinnerPreset::Dots2 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS2,
            },
            #[cfg(feature = "dots3")]
            SpinnerPreset::Dots3 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS3,
            },
            #[cfg(feature = "dots4")]
            SpinnerPreset::Dots4 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS4,
            },
            #[cfg(feature = "dots5")]
            SpinnerPreset::Dots5 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS5,
            },
            #[cfg(feature = "dots6")]
            SpinnerPreset::Dots6 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS6,
            },
            #[cfg(feature = "dots7")]
            SpinnerPreset::Dots7 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS7,
            },
            #[cfg(feature = "dots8")]
            SpinnerPreset::Dots8 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS8,
            },
            #[cfg(feature = "dots8_bit")]
            SpinnerPreset::Dots8Bit => SpinnerFrames {
                interval: ms(80),
                frames: DOTS8_BIT,
            },
            #[cfg(feature = "dots9")]
            SpinnerPreset::Dots9 => SpinnerFrames {
                interval: ms(80),
                frames: DOTS9,
            },
            #[cfg(feature = "dots_circle")]
            SpinnerPreset::DotsCircle => SpinnerFrames {
                interval: ms(80),
                frames: DOTS_CIRCLE,
            },
            #[cfg(feature = "dqpb")]
            SpinnerPreset::Dqpb => SpinnerFrames {
                interval: ms(100),
                frames: DQPB,
            },
            #[cfg(feature = "dwarf_fortress")]
            SpinnerPreset::DwarfFortress => SpinnerFrames {
                interval: ms(80),
                frames: DWARF_FORTRESS,
            },
            #[cfg(feature = "earth")]
            SpinnerPreset::Earth => SpinnerFrames {
                interval: ms(180),
                frames: EARTH,
            },
            #[cfg(feature = "finger_dance")]
            SpinnerPreset::FingerDance => SpinnerFrames {
                interval: ms(160),
                frames: FINGER_DANCE,
            },
            #[cfg(feature = "fist_bump")]
            SpinnerPreset::FistBump => SpinnerFrames {
                interval: ms(80),
                frames: FIST_BUMP,
            },
            #[cfg(feature = "flip")]
            SpinnerPreset::Flip => SpinnerFrames {
                interval: ms(70),
                frames: FLIP,
            },
            #[cfg(feature = "grenade")]
            SpinnerPreset::Grenade => SpinnerFrames {
                interval: ms(80),
                frames: GRENADE,
            },
            #[cfg(feature = "grow_horizontal")]
            SpinnerPreset::GrowHorizontal => SpinnerFrames {
                interval: ms(120),
                frames: GROW_HORIZONTAL,
            },
            #[cfg(feature = "grow_vertical")]
            SpinnerPreset::GrowVertical => SpinnerFrames {
                interval: ms(120),
                frames: GROW_VERTICAL,
            },
            #[cfg(feature = "hamburger")]
            SpinnerPreset::Hamburger => SpinnerFrames {
                interval: ms(100),
                frames: HAMBURGER,
            },
            #[cfg(feature = "hearts")]
            SpinnerPreset::Hearts => SpinnerFrames {
                interval: ms(100),
                frames: HEARTS,
            },
            #[cfg(feature = "layer")]
            SpinnerPreset::Layer => SpinnerFrames {
                interval: ms(150),
                frames: LAYER,
            },

            #[cfg(feature = "line2")]
            SpinnerPreset::Line2 => SpinnerFrames {
                interval: ms(100),
                frames: LINE2,
            },
            #[cfg(feature = "material")]
            SpinnerPreset::Material => SpinnerFrames {
                interval: ms(17),
                frames: MATERIAL,
            },
            #[cfg(feature = "mindblown")]
            SpinnerPreset::Mindblown => SpinnerFrames {
                interval: ms(160),
                frames: MINDBLOWN,
            },
            #[cfg(feature = "monkey")]
            SpinnerPreset::Monkey => SpinnerFrames {
                interval: ms(300),
                frames: MONKEY,
            },
            #[cfg(feature = "moon")]
            SpinnerPreset::Moon => SpinnerFrames {
                interval: ms(80),
                frames: MOON,
            },
            #[cfg(feature = "noise")]
            SpinnerPreset::Noise => SpinnerFrames {
                interval: ms(100),
                frames: NOISE,
            },
            #[cfg(feature = "orange_blue_pulse")]
            SpinnerPreset::OrangeBluePulse => SpinnerFrames {
                interval: ms(100),
                frames: ORANGE_BLUE_PULSE,
            },
            #[cfg(feature = "orange_pulse")]
            SpinnerPreset::OrangePulse => SpinnerFrames {
                interval: ms(100),
                frames: ORANGE_PULSE,
            },
            #[cfg(feature = "pipe")]
            SpinnerPreset::Pipe => SpinnerFrames {
                interval: ms(100),
                frames: PIPE,
            },
            #[cfg(feature = "point")]
            SpinnerPreset::Point => SpinnerFrames {
                interval: ms(125),
                frames: POINT,
            },
            #[cfg(feature = "pong")]
            SpinnerPreset::Pong => SpinnerFrames {
                interval: ms(80),
                frames: PONG,
            },
            #[cfg(feature = "runner")]
            SpinnerPreset::Runner => SpinnerFrames {
                interval: ms(140),
                frames: RUNNER,
            },
            #[cfg(feature = "sand")]
            SpinnerPreset::Sand => SpinnerFrames {
                interval: ms(80),
                frames: SAND,
            },
            #[cfg(feature = "shark")]
            SpinnerPreset::Shark => SpinnerFrames {
                interval: ms(120),
                frames: SHARK,
            },
            #[cfg(feature = "simple_dots")]
            SpinnerPreset::SimpleDots => SpinnerFrames {
                interval: ms(400),
                frames: SIMPLE_DOTS,
            },
            #[cfg(feature = "simple_dots_scrolling")]
            SpinnerPreset::SimpleDotsScrolling => SpinnerFrames {
                interval: ms(200),
                frames: SIMPLE_DOTS_SCROLLING,
            },
            #[cfg(feature = "smiley")]
            SpinnerPreset::Smiley => SpinnerFrames {
                interval: ms(200),
                frames: SMILEY,
            },
            #[cfg(feature = "soccer_header")]
            SpinnerPreset::SoccerHeader => SpinnerFrames {
                interval: ms(80),
                frames: SOCCER_HEADER,
            },
            #[cfg(feature = "speaker")]
            SpinnerPreset::Speaker => SpinnerFrames {
                interval: ms(160),
                frames: SPEAKER,
            },
            #[cfg(feature = "square_corners")]
            SpinnerPreset::SquareCorners => SpinnerFrames {
                interval: ms(180),
                frames: SQUARE_CORNERS,
            },
            #[cfg(feature = "squish")]
            SpinnerPreset::Squish => SpinnerFrames {
                interval: ms(100),
                frames: SQUISH,
            },
            #[cfg(feature = "star")]
            SpinnerPreset::Star => SpinnerFrames {
                interval: ms(70),
                frames: STAR,
            },
            #[cfg(feature = "star2")]
            SpinnerPreset::Star2 => SpinnerFrames {
                interval: ms(80),
                frames: STAR2,
            },
            #[cfg(feature = "time_travel")]
            SpinnerPreset::TimeTravel => SpinnerFrames {
                interval: ms(100),
                frames: TIME_TRAVEL,
            },

            #[cfg(feature = "toggle10")]
            SpinnerPreset::Toggle10 => SpinnerFrames {
                interval: ms(100),
                frames: TOGGLE10,
            },
            #[cfg(feature = "toggle11")]
            SpinnerPreset::Toggle11 => SpinnerFrames {
                interval: ms(50),
                frames: TOGGLE11,
            },
            #[cfg(feature = "toggle12")]
            SpinnerPreset::Toggle12 => SpinnerFrames {
                interval: ms(120),
                frames: TOGGLE12,
            },
            #[cfg(feature = "toggle13")]
            SpinnerPreset::Toggle13 => SpinnerFrames {
                interval: ms(80),
                frames: TOGGLE13,
            },
            #[cfg(feature = "toggle2")]
            SpinnerPreset::Toggle2 => SpinnerFrames {
                interval: ms(80),
                frames: TOGGLE2,
            },
            #[cfg(feature = "toggle3")]
            SpinnerPreset::Toggle3 => SpinnerFrames {
                interval: ms(120),
                frames: TOGGLE3,
            },
            #[cfg(feature = "toggle4")]
            SpinnerPreset::Toggle4 => SpinnerFrames {
                interval: ms(100),
                frames: TOGGLE4,
            },
            #[cfg(feature = "toggle5")]
            SpinnerPreset::Toggle5 => SpinnerFrames {
                interval: ms(100),
                frames: TOGGLE5,
            },
            #[cfg(feature = "toggle6")]
            SpinnerPreset::Toggle6 => SpinnerFrames {
                interval: ms(300),
                frames: TOGGLE6,
            },
            #[cfg(feature = "toggle7")]
            SpinnerPreset::Toggle7 => SpinnerFrames {
                interval: ms(80),
                frames: TOGGLE7,
            },
            #[cfg(feature = "toggle8")]
            SpinnerPreset::Toggle8 => SpinnerFrames {
                interval: ms(100),
                frames: TOGGLE8,
            },
            #[cfg(feature = "toggle9")]
            SpinnerPreset::Toggle9 => SpinnerFrames {
                interval: ms(100),
                frames: TOGGLE9,
            },
            #[cfg(feature = "triangle")]
            SpinnerPreset::Triangle => SpinnerFrames {
                interval: ms(50),
                frames: TRIANGLE,
            },
            #[cfg(feature = "weather")]
            SpinnerPreset::Weather => SpinnerFrames {
                interval: ms(100),
                frames: WEATHER,
            },
        }
    }

    /// Convenience - update period
    #[inline]
    pub const fn interval(self) -> Duration {
        self.data().interval
    }

    /// Convenience - frame table
    #[inline]
    pub const fn frames(self) -> &'static [&'static str] {
        self.data().frames
    }
}
