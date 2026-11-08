//! What the language server knows about the language: keywords, units,
//! built-in classes, objects and functions, and code snippets.
//!
//! Every text exists in English and German. The contents follow
//! `docs/language.md` in the Core repository; update both together.

use ekips_i18n::{Lang, t};

/// What kind of thing an [`Entry`] describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A reserved word such as `while`
    Keyword,
    /// A unit suffix such as `cm`
    Unit,
    /// A class used with `new`, such as `Motor`
    Class,
    /// A built-in object such as `hub` or `Port`
    Object,
    /// A global function such as `wait`
    Function,
    /// A method of a class or object, such as `drive`
    Method,
    /// A property of a class or object, such as `angle`
    Property,
    /// A fixed value such as `Port.A`
    Constant,
}

/// One documented item.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// Name as written in the source
    pub name: &'static str,
    /// What it is
    pub kind: Kind,
    /// How it is used, e.g. `drive(distance, speed?)`
    pub signature: &'static str,
    /// English description (Markdown)
    pub en: &'static str,
    /// German description (Markdown)
    pub de: &'static str,
}

impl Entry {
    /// Description in the given language.
    pub fn doc(&self, lang: Lang) -> &'static str {
        t(lang, self.en, self.de)
    }

    /// Hover text: signature as code block plus description.
    pub fn hover_markdown(&self, owner: Option<&str>, lang: Lang) -> String {
        let signature = match (owner, self.kind) {
            (Some(owner), Kind::Constant) => format!("{owner}.{}", self.signature),
            (Some(owner), _) if owner.starts_with(char::is_lowercase) => {
                format!("{owner}.{}", self.signature)
            }
            _ => self.signature.to_string(),
        };
        format!("```ts\n{signature}\n```\n\n{}", self.doc(lang))
    }
}

/// Members of a class or built-in object.
#[derive(Debug, Clone, Copy)]
pub struct Group {
    /// Class or object name, e.g. `Motor` or `hub`
    pub owner: &'static str,
    /// Its methods, properties and constants
    pub members: &'static [Entry],
}

const fn e(
    name: &'static str,
    kind: Kind,
    signature: &'static str,
    en: &'static str,
    de: &'static str,
) -> Entry {
    Entry {
        name,
        kind,
        signature,
        en,
        de,
    }
}

use Kind::*;

/// All reserved words.
pub const KEYWORDS: &[Entry] = &[
    e(
        "async",
        Keyword,
        "async function name() { … }",
        "Marks a function that may use `await`, so it can wait for movements. `main` must be `async`.",
        "Markiert eine Funktion, in der `await` benutzt werden darf, damit sie auf Bewegungen warten kann. `main` muss `async` sein.",
    ),
    e(
        "await",
        Keyword,
        "await promise",
        "Waits until a command that takes time is finished, e.g. `await robot.drive(20cm);`. Without `await` the next line starts immediately.",
        "Wartet, bis ein Befehl, der Zeit braucht, fertig ist, z. B. `await robot.drive(20cm);`. Ohne `await` startet die nächste Zeile sofort.",
    ),
    e(
        "break",
        Keyword,
        "break;",
        "Leaves the loop immediately.",
        "Verlässt die Schleife sofort.",
    ),
    e(
        "const",
        Keyword,
        "const name = value;",
        "Creates a name for a value that never changes, e.g. `const arm = new Motor(Port.C);`.",
        "Legt einen Namen für einen Wert an, der sich nie ändert, z. B. `const arm = new Motor(Port.C);`.",
    ),
    e(
        "continue",
        Keyword,
        "continue;",
        "Skips the rest of this round and starts the next round of the loop.",
        "Überspringt den Rest dieser Runde und beginnt die nächste Runde der Schleife.",
    ),
    e(
        "else",
        Keyword,
        "if (condition) { … } else { … }",
        "The code after `else` runs when the `if` condition is `false`.",
        "Der Code nach `else` läuft, wenn die `if`-Bedingung `false` ist.",
    ),
    e(
        "false",
        Keyword,
        "false",
        "The value \"no\" / \"not true\".",
        "Der Wert \"nein\" / \"nicht wahr\".",
    ),
    e(
        "for",
        Keyword,
        "for (let i = 0; i < 4; i++) { … }\nfor (const x of list) { … }",
        "Repeats code: a fixed number of times, or once for every element of a list.",
        "Wiederholt Code: eine feste Anzahl von Malen, oder einmal für jedes Element einer Liste.",
    ),
    e(
        "function",
        Keyword,
        "function name(parameters) { … }",
        "Defines a function: a named piece of code you can run again and again. Write `async` in front if it uses `await`.",
        "Definiert eine Funktion: ein Stück Code mit Namen, das du immer wieder ausführen kannst. Schreibe `async` davor, wenn sie `await` benutzt.",
    ),
    e(
        "if",
        Keyword,
        "if (condition) { … }",
        "Runs the code only if the condition is `true`.",
        "Führt den Code nur aus, wenn die Bedingung `true` ist.",
    ),
    e(
        "let",
        Keyword,
        "let name = value;",
        "Creates a variable that can change later, e.g. `let count = 0;`.",
        "Legt eine Variable an, die sich später ändern kann, z. B. `let count = 0;`.",
    ),
    e(
        "new",
        Keyword,
        "new Motor(Port.A)",
        "Creates a new object, e.g. a motor or sensor at a port.",
        "Erzeugt ein neues Objekt, z. B. einen Motor oder Sensor an einem Port.",
    ),
    e(
        "null",
        Keyword,
        "null",
        "The value \"nothing\".",
        "Der Wert \"nichts\".",
    ),
    e(
        "of",
        Keyword,
        "for (const x of list) { … }",
        "Used in `for … of` loops: goes through every element of a list.",
        "Wird in `for … of`-Schleifen benutzt: geht jedes Element einer Liste durch.",
    ),
    e(
        "return",
        Keyword,
        "return value;",
        "Leaves the function and gives back a value.",
        "Verlässt die Funktion und gibt einen Wert zurück.",
    ),
    e(
        "true",
        Keyword,
        "true",
        "The value \"yes\" / \"true\".",
        "Der Wert \"ja\" / \"wahr\".",
    ),
    e(
        "undefined",
        Keyword,
        "undefined",
        "The value of a variable that has not been given a value yet.",
        "Der Wert einer Variable, der noch kein Wert gegeben wurde.",
    ),
    e(
        "while",
        Keyword,
        "while (condition) { … }",
        "Repeats the code as long as the condition is `true`.",
        "Wiederholt den Code, solange die Bedingung `true` ist.",
    ),
];

/// Unit suffixes for numbers.
pub const UNITS: &[Entry] = &[
    e(
        "mm",
        Unit,
        "150mm",
        "Millimetres (length).",
        "Millimeter (Länge).",
    ),
    e(
        "cm",
        Unit,
        "20cm",
        "Centimetres (length). `1cm` is `10mm`.",
        "Zentimeter (Länge). `1cm` sind `10mm`.",
    ),
    e(
        "deg",
        Unit,
        "90deg",
        "Degrees (angle). You can also write `90°`.",
        "Grad (Winkel). Du kannst auch `90°` schreiben.",
    ),
    e(
        "rot",
        Unit,
        "1.5rot",
        "Full rotations (angle). `1rot` is `360deg`.",
        "Volle Umdrehungen (Winkel). `1rot` sind `360deg`.",
    ),
    e("s", Unit, "2s", "Seconds (time).", "Sekunden (Zeit)."),
    e(
        "ms",
        Unit,
        "300ms",
        "Milliseconds (time). `1s` is `1000ms`.",
        "Millisekunden (Zeit). `1s` sind `1000ms`.",
    ),
    e(
        "%",
        Unit,
        "50%",
        "Percent, used for speed and power. Between two values `%` is the remainder: `10 % 3` is `1`.",
        "Prozent, für Geschwindigkeit und Kraft. Zwischen zwei Werten ist `%` der Rest: `10 % 3` ist `1`.",
    ),
];

/// Global names: classes, objects and functions.
pub const GLOBALS: &[Entry] = &[
    e(
        "Motor",
        Class,
        "new Motor(port)",
        "A motor at one of the hub's ports, e.g. `new Motor(Port.C)`.",
        "Ein Motor an einem Port des Hubs, z. B. `new Motor(Port.C)`.",
    ),
    e(
        "DriveBase",
        Class,
        "new DriveBase(leftMotor, rightMotor, wheelDiameter, axleTrack)",
        "Two motors that drive the robot together. `wheelDiameter` is the diameter of the wheels, `axleTrack` the distance between them, e.g. `56mm` and `112mm`.",
        "Zwei Motoren, die den Roboter gemeinsam fahren. `wheelDiameter` ist der Durchmesser der Räder, `axleTrack` der Abstand zwischen ihnen, z. B. `56mm` und `112mm`.",
    ),
    e(
        "ColorSensor",
        Class,
        "new ColorSensor(port)",
        "A color sensor. It sees colors and measures reflected light.",
        "Ein Farbsensor. Er erkennt Farben und misst das zurückgeworfene Licht.",
    ),
    e(
        "DistanceSensor",
        Class,
        "new DistanceSensor(port)",
        "An ultrasonic distance sensor. It measures how far away something is.",
        "Ein Ultraschall-Abstandssensor. Er misst, wie weit etwas entfernt ist.",
    ),
    e(
        "ForceSensor",
        Class,
        "new ForceSensor(port)",
        "A force (touch) sensor. It knows if and how hard it is pressed.",
        "Ein Kraftsensor (Taster). Er weiß, ob und wie fest er gedrückt wird.",
    ),
    e(
        "Port",
        Object,
        "Port.A … Port.F",
        "The six ports of the hub, `Port.A` to `Port.F`.",
        "Die sechs Ports des Hubs, `Port.A` bis `Port.F`.",
    ),
    e(
        "Color",
        Object,
        "Color.Black, Color.Red, …",
        "The colors the color sensor can see.",
        "Die Farben, die der Farbsensor erkennen kann.",
    ),
    e(
        "hub",
        Object,
        "hub",
        "The SPIKE Prime hub itself: light matrix, speaker, gyro sensor, timer and battery.",
        "Der SPIKE-Prime-Hub selbst: Lichtmatrix, Lautsprecher, Gyrosensor, Stoppuhr und Akku.",
    ),
    e(
        "Promise",
        Object,
        "Promise.all([…]), Promise.race([…])",
        "Helps doing several things at the same time.",
        "Hilft, mehrere Dinge gleichzeitig zu tun.",
    ),
    e(
        "console",
        Object,
        "console.log(…)",
        "Prints messages to the computer (USB console or simulator).",
        "Gibt Nachrichten auf dem Computer aus (USB-Konsole oder Simulator).",
    ),
    e(
        "Math",
        Object,
        "Math.round(x), Math.PI, …",
        "Math functions and constants.",
        "Mathe-Funktionen und Konstanten.",
    ),
    e(
        "wait",
        Function,
        "wait(time)",
        "Returns a Promise that finishes after `time`, e.g. `await wait(2s);`.",
        "Gibt ein Promise zurück, das nach `time` fertig ist, z. B. `await wait(2s);`.",
    ),
    e(
        "waitUntil",
        Function,
        "waitUntil(() => condition)",
        "Returns a Promise that finishes as soon as the condition is `true`, e.g. `await waitUntil(() => bumper.pressed);`.",
        "Gibt ein Promise zurück, das fertig ist, sobald die Bedingung `true` ist, z. B. `await waitUntil(() => bumper.pressed);`.",
    ),
];

const MOTOR: &[Entry] = &[
    e(
        "rotate",
        Method,
        "rotate(angle, speed?)",
        "Rotates by an angle (`deg` or `rot`). Returns a Promise, so use `await` to wait for it.",
        "Dreht um einen Winkel (`deg` oder `rot`). Gibt ein Promise zurück, also mit `await` darauf warten.",
    ),
    e(
        "run",
        Method,
        "run(speed)",
        "Starts rotating until `stop()`. Negative speed = other direction.",
        "Beginnt zu drehen, bis `stop()` kommt. Negative Geschwindigkeit = andere Richtung.",
    ),
    e(
        "stop",
        Method,
        "stop()",
        "Stops and holds the position.",
        "Hält an und hält die Position.",
    ),
    e(
        "angle",
        Property,
        "angle",
        "Angle since the last `resetAngle()` (in `deg`).",
        "Winkel seit dem letzten `resetAngle()` (in `deg`).",
    ),
    e(
        "speed",
        Property,
        "speed",
        "Current speed (in `%`).",
        "Aktuelle Geschwindigkeit (in `%`).",
    ),
    e(
        "resetAngle",
        Method,
        "resetAngle()",
        "Sets `angle` to `0deg`.",
        "Setzt `angle` auf `0deg`.",
    ),
];

const DRIVE_BASE: &[Entry] = &[
    e(
        "drive",
        Method,
        "drive(distance, speed?)",
        "Drives straight. Negative distance = backwards. Returns a Promise, so use `await` to wait for it.",
        "Fährt geradeaus. Negative Strecke = rückwärts. Gibt ein Promise zurück, also mit `await` darauf warten.",
    ),
    e(
        "turn",
        Method,
        "turn(angle, speed?)",
        "Turns on the spot. Positive = right (clockwise). Returns a Promise.",
        "Dreht sich auf der Stelle. Positiv = rechts (im Uhrzeigersinn). Gibt ein Promise zurück.",
    ),
    e(
        "run",
        Method,
        "run(speed)",
        "Starts driving straight until `stop()`.",
        "Beginnt geradeaus zu fahren, bis `stop()` kommt.",
    ),
    e(
        "stop",
        Method,
        "stop()",
        "Stops both wheels.",
        "Hält beide Räder an.",
    ),
];

const COLOR_SENSOR: &[Entry] = &[
    e(
        "color",
        Property,
        "color",
        "The color seen right now, e.g. `Color.Red`.",
        "Die Farbe, die gerade erkannt wird, z. B. `Color.Red`.",
    ),
    e(
        "sees",
        Method,
        "sees(color)",
        "`true` if the sensor sees this color.",
        "`true`, wenn der Sensor diese Farbe sieht.",
    ),
    e(
        "reflection",
        Property,
        "reflection",
        "Reflected light in `%` (dark = low, bright = high).",
        "Zurückgeworfenes Licht in `%` (dunkel = wenig, hell = viel).",
    ),
];

const DISTANCE_SENSOR: &[Entry] = &[e(
    "distance",
    Property,
    "distance",
    "Distance as a length; `200cm` if nothing is in range.",
    "Abstand als Länge; `200cm`, wenn nichts in Reichweite ist.",
)];

const FORCE_SENSOR: &[Entry] = &[
    e(
        "pressed",
        Property,
        "pressed",
        "`true` while the sensor is pressed.",
        "`true`, solange der Sensor gedrückt wird.",
    ),
    e(
        "force",
        Property,
        "force",
        "Force in newtons (plain number).",
        "Kraft in Newton (Zahl ohne Einheit).",
    ),
];

const PORT: &[Entry] = &[
    e("A", Constant, "A", "Port A of the hub.", "Port A des Hubs."),
    e("B", Constant, "B", "Port B of the hub.", "Port B des Hubs."),
    e("C", Constant, "C", "Port C of the hub.", "Port C des Hubs."),
    e("D", Constant, "D", "Port D of the hub.", "Port D des Hubs."),
    e("E", Constant, "E", "Port E of the hub.", "Port E des Hubs."),
    e("F", Constant, "F", "Port F of the hub.", "Port F des Hubs."),
];

const COLOR: &[Entry] = &[
    e(
        "None",
        Constant,
        "None",
        "No color (nothing in front of the sensor).",
        "Keine Farbe (nichts vor dem Sensor).",
    ),
    e("Black", Constant, "Black", "Black.", "Schwarz."),
    e("Violet", Constant, "Violet", "Violet.", "Violett."),
    e("Blue", Constant, "Blue", "Blue.", "Blau."),
    e(
        "Azure",
        Constant,
        "Azure",
        "Azure (light blue).",
        "Azur (Hellblau).",
    ),
    e("Green", Constant, "Green", "Green.", "Grün."),
    e("Yellow", Constant, "Yellow", "Yellow.", "Gelb."),
    e("Red", Constant, "Red", "Red.", "Rot."),
    e("White", Constant, "White", "White.", "Weiß."),
];

const HUB: &[Entry] = &[
    e(
        "show",
        Method,
        "show(value)",
        "Shows a number or short text on the light matrix.",
        "Zeigt eine Zahl oder einen kurzen Text auf der Lichtmatrix.",
    ),
    e(
        "beep",
        Method,
        "beep(frequency?, duration?)",
        "Beeps. Returns a Promise.",
        "Piept. Gibt ein Promise zurück.",
    ),
    e(
        "heading",
        Property,
        "heading",
        "Gyro heading since `resetHeading()` (in `deg`, right = positive).",
        "Gyro-Richtung seit `resetHeading()` (in `deg`, rechts = positiv).",
    ),
    e(
        "resetHeading",
        Method,
        "resetHeading()",
        "Sets the heading to `0deg`.",
        "Setzt die Richtung auf `0deg`.",
    ),
    e(
        "timer",
        Property,
        "timer",
        "Time since the start or `resetTimer()` (in `s`).",
        "Zeit seit dem Start oder `resetTimer()` (in `s`).",
    ),
    e(
        "resetTimer",
        Method,
        "resetTimer()",
        "Sets the timer to `0s`.",
        "Setzt die Stoppuhr auf `0s`.",
    ),
    e(
        "battery",
        Property,
        "battery",
        "Battery charge in `%`.",
        "Akkuladung in `%`.",
    ),
];

const PROMISE: &[Entry] = &[
    e(
        "all",
        Method,
        "all([a, b, …])",
        "Finishes when **all** given Promises are done. Use it to do things at the same time.",
        "Ist fertig, wenn **alle** angegebenen Promises fertig sind. Damit machst du Dinge gleichzeitig.",
    ),
    e(
        "race",
        Method,
        "race([a, b, …])",
        "Finishes when the **first** given Promise is done, e.g. drive until a line is seen.",
        "Ist fertig, sobald das **erste** angegebene Promise fertig ist, z. B. fahren, bis eine Linie erkannt wird.",
    ),
];

const CONSOLE: &[Entry] = &[e(
    "log",
    Method,
    "log(a, b, …)",
    "Prints the values to the computer.",
    "Gibt die Werte auf dem Computer aus.",
)];

const MATH: &[Entry] = &[
    e(
        "abs",
        Method,
        "abs(x)",
        "Value without sign: `Math.abs(-3)` is `3`.",
        "Betrag ohne Vorzeichen: `Math.abs(-3)` ist `3`.",
    ),
    e(
        "min",
        Method,
        "min(a, b, …)",
        "The smallest value.",
        "Der kleinste Wert.",
    ),
    e(
        "max",
        Method,
        "max(a, b, …)",
        "The largest value.",
        "Der größte Wert.",
    ),
    e(
        "round",
        Method,
        "round(x)",
        "Rounds to the nearest whole number.",
        "Rundet auf die nächste ganze Zahl.",
    ),
    e("floor", Method, "floor(x)", "Rounds down.", "Rundet ab."),
    e("ceil", Method, "ceil(x)", "Rounds up.", "Rundet auf."),
    e("sqrt", Method, "sqrt(x)", "Square root.", "Quadratwurzel."),
    e(
        "random",
        Method,
        "random()",
        "A random number from 0 (included) to 1 (not included).",
        "Eine Zufallszahl von 0 (eingeschlossen) bis 1 (ausgeschlossen).",
    ),
    e(
        "PI",
        Constant,
        "PI",
        "The number π (about 3.14159).",
        "Die Zahl π (ungefähr 3,14159).",
    ),
];

/// Members of every class and built-in object.
pub const GROUPS: &[Group] = &[
    Group {
        owner: "Motor",
        members: MOTOR,
    },
    Group {
        owner: "DriveBase",
        members: DRIVE_BASE,
    },
    Group {
        owner: "ColorSensor",
        members: COLOR_SENSOR,
    },
    Group {
        owner: "DistanceSensor",
        members: DISTANCE_SENSOR,
    },
    Group {
        owner: "ForceSensor",
        members: FORCE_SENSOR,
    },
    Group {
        owner: "Port",
        members: PORT,
    },
    Group {
        owner: "Color",
        members: COLOR,
    },
    Group {
        owner: "hub",
        members: HUB,
    },
    Group {
        owner: "Promise",
        members: PROMISE,
    },
    Group {
        owner: "console",
        members: CONSOLE,
    },
    Group {
        owner: "Math",
        members: MATH,
    },
];

/// Looks up a keyword or global name.
pub fn global(name: &str) -> Option<&'static Entry> {
    KEYWORDS.iter().chain(GLOBALS).find(|e| e.name == name)
}

/// Looks up a unit suffix (`°` is the same as `deg`).
pub fn unit(suffix: &str) -> Option<&'static Entry> {
    let suffix = if suffix == "°" { "deg" } else { suffix };
    UNITS.iter().find(|e| e.name == suffix)
}

/// Members of a class or built-in object.
pub fn members(owner: &str) -> Option<&'static [Entry]> {
    GROUPS.iter().find(|g| g.owner == owner).map(|g| g.members)
}

/// Looks up `owner.name`.
pub fn member(owner: &str, name: &str) -> Option<&'static Entry> {
    members(owner)?.iter().find(|e| e.name == name)
}

/// A code template offered by autocompletion.
#[derive(Debug, Clone, Copy)]
pub struct Snippet {
    /// What the student types to find it
    pub label: &'static str,
    /// Short English description
    pub en: &'static str,
    /// Short German description
    pub de: &'static str,
    /// LSP snippet syntax (`$1`, `${1:name}`, `$0`)
    pub body: &'static str,
}

impl Snippet {
    /// Description in the given language.
    pub fn detail(&self, lang: Lang) -> &'static str {
        t(lang, self.en, self.de)
    }

    /// The body with all placeholders replaced by their default text, for
    /// showing as preview and for editors without snippet support.
    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        let mut chars = self.body.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '$' {
                out.push(c);
                continue;
            }
            match chars.peek() {
                Some('{') => {
                    chars.next();
                    // Skip the tab stop number.
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        chars.next();
                    }
                    let mut inner = String::new();
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                        inner.push(c);
                    }
                    if let Some(text) = inner.strip_prefix(':') {
                        out.push_str(text);
                    } else if let Some(choices) = inner.strip_prefix('|') {
                        out.push_str(choices.split([',', '|']).next().unwrap_or(""));
                    }
                }
                Some(d) if d.is_ascii_digit() => {
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        chars.next();
                    }
                }
                _ => out.push('$'),
            }
        }
        out
    }
}

/// All snippets.
pub const SNIPPETS: &[Snippet] = &[
    Snippet {
        label: "main",
        en: "The main function: the program starts here",
        de: "Die main-Funktion: hier startet das Programm",
        body: "async function main() {\n\t$0\n}",
    },
    Snippet {
        label: "program",
        en: "A complete program with a drive base",
        de: "Ein vollständiges Programm mit Fahrgestell",
        body: "const leftWheel = new Motor(Port.${1|A,B,C,D,E,F|});\nconst rightWheel = new Motor(Port.${2|B,A,C,D,E,F|});\nconst robot = new DriveBase(leftWheel, rightWheel, ${3:56mm}, ${4:112mm});\n\nasync function main() {\n\t${0:await robot.drive(20cm);}\n}",
    },
    Snippet {
        label: "function",
        en: "A function",
        de: "Eine Funktion",
        body: "function ${1:name}($2) {\n\t$0\n}",
    },
    Snippet {
        label: "async function",
        en: "A function that can wait with `await`",
        de: "Eine Funktion, die mit `await` warten kann",
        body: "async function ${1:name}($2) {\n\t$0\n}",
    },
    Snippet {
        label: "if",
        en: "Do something only if a condition is true",
        de: "Etwas nur tun, wenn eine Bedingung wahr ist",
        body: "if (${1:condition}) {\n\t$0\n}",
    },
    Snippet {
        label: "if else",
        en: "Do one thing or the other",
        de: "Das eine oder das andere tun",
        body: "if (${1:condition}) {\n\t$2\n} else {\n\t$0\n}",
    },
    Snippet {
        label: "while",
        en: "Repeat as long as a condition is true",
        de: "Wiederholen, solange eine Bedingung wahr ist",
        body: "while (${1:condition}) {\n\t$0\n}",
    },
    Snippet {
        label: "for",
        en: "Repeat a fixed number of times",
        de: "Eine feste Anzahl von Malen wiederholen",
        body: "for (let ${1:i} = 0; ${1:i} < ${2:4}; ${1:i}++) {\n\t$0\n}",
    },
    Snippet {
        label: "for of",
        en: "Repeat for every element of a list",
        de: "Für jedes Element einer Liste wiederholen",
        body: "for (const ${1:item} of ${2:list}) {\n\t$0\n}",
    },
    Snippet {
        label: "motor",
        en: "Set up a motor",
        de: "Einen Motor einrichten",
        body: "const ${1:arm} = new Motor(Port.${2|A,B,C,D,E,F|});",
    },
    Snippet {
        label: "drivebase",
        en: "Set up two motors as a drive base",
        de: "Zwei Motoren als Fahrgestell einrichten",
        body: "const ${1:robot} = new DriveBase(${2:leftWheel}, ${3:rightWheel}, ${4:56mm}, ${5:112mm});",
    },
    Snippet {
        label: "colorsensor",
        en: "Set up a color sensor",
        de: "Einen Farbsensor einrichten",
        body: "const ${1:eye} = new ColorSensor(Port.${2|A,B,C,D,E,F|});",
    },
    Snippet {
        label: "distancesensor",
        en: "Set up a distance sensor",
        de: "Einen Abstandssensor einrichten",
        body: "const ${1:sonar} = new DistanceSensor(Port.${2|A,B,C,D,E,F|});",
    },
    Snippet {
        label: "forcesensor",
        en: "Set up a force sensor",
        de: "Einen Kraftsensor einrichten",
        body: "const ${1:bumper} = new ForceSensor(Port.${2|A,B,C,D,E,F|});",
    },
    Snippet {
        label: "all",
        en: "Do several things at the same time",
        de: "Mehrere Dinge gleichzeitig tun",
        body: "await Promise.all([\n\t${1:robot.drive(30cm)},\n\t${2:arm.rotate(90deg)},\n]);",
    },
    Snippet {
        label: "race",
        en: "Drive until something happens",
        de: "Fahren, bis etwas passiert",
        body: "await Promise.race([\n\t${1:robot.drive(100cm)},\n\twaitUntil(() => ${2:eye.sees(Color.Black)}),\n]);\n${3:robot}.stop();",
    },
    Snippet {
        label: "waitUntil",
        en: "Wait until a condition is true",
        de: "Warten, bis eine Bedingung wahr ist",
        body: "await waitUntil(() => ${1:bumper.pressed});",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_keyword_is_documented() {
        for kw in ekips_compiler::keywords::Keyword::ALL {
            assert!(
                KEYWORDS.iter().any(|e| e.name == kw.as_str()),
                "keyword `{}` has no documentation",
                kw.as_str()
            );
        }
    }

    #[test]
    fn every_unit_is_documented() {
        for suffix in ["mm", "cm", "deg", "°", "rot", "s", "ms", "%"] {
            assert!(
                ekips_compiler::ast::Unit::from_suffix(suffix).is_some(),
                "{suffix}"
            );
            assert!(
                unit(suffix).is_some(),
                "unit `{suffix}` has no documentation"
            );
        }
    }

    #[test]
    fn every_global_object_has_members() {
        for entry in GLOBALS.iter().filter(|e| matches!(e.kind, Class | Object)) {
            assert!(members(entry.name).is_some(), "{}", entry.name);
        }
    }

    #[test]
    fn texts_are_not_empty() {
        let all = KEYWORDS
            .iter()
            .chain(UNITS)
            .chain(GLOBALS)
            .chain(GROUPS.iter().flat_map(|g| g.members));
        for entry in all {
            assert!(
                !entry.en.is_empty() && !entry.de.is_empty(),
                "{}",
                entry.name
            );
        }
        for s in SNIPPETS {
            assert!(!s.en.is_empty() && !s.de.is_empty(), "{}", s.label);
        }
    }

    #[test]
    fn snippets_are_valid_programs() {
        for snippet in SNIPPETS {
            let code = snippet.plain_text();
            // Wrap statements in an async main so `await` etc. are allowed.
            let source = if code.contains("function main") {
                code.clone()
            } else if code.starts_with("const ") || code.contains("function ") {
                format!("{code}\nasync function main() {{}}\n")
            } else {
                format!("async function main() {{\nfor (;;) {{\n{code}\n}}\n}}\n")
            };
            let result = ekips_compiler::parse(&source);
            assert!(
                result.diagnostics.is_empty(),
                "snippet `{}` does not parse:\n{source}\n{:?}",
                snippet.label,
                result.diagnostics
            );
        }
    }

    #[test]
    fn plain_text_resolves_placeholders() {
        let s = Snippet {
            label: "x",
            en: "x",
            de: "x",
            body: "const ${1:arm} = new Motor(Port.${2|A,B|});$0",
        };
        assert_eq!(s.plain_text(), "const arm = new Motor(Port.A);");
    }
}
