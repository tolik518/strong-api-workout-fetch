<?php
/**
 * Workout Heatmap (PNG)
 *
 * Renders a GitHub-style contribution heatmap of training tonnage (weight × reps)
 * from a ClickHouse database. Output is a PNG image with transparent or solid background.
 *
 * -----------------------------------------------------------------------------
 * QUERY PARAMS
 * -----------------------------------------------------------------------------
 *
 * Style & Colors:
 *   ?style=dark              Theme. Default: dark
 *                            Options: dark, light, material_dark, halloween,
 *                                     dracula, solarized, nord, monokai,
 *                                     gruvbox, one_dark, catppuccin, tokyo_night
 *   ?bg=<hex|transparent>    Override canvas background color.
 *                            e.g. ?bg=0d1117  or  ?bg=transparent
 *   ?empty=<hex|transparent> Override empty-cell fill color.
 *                            e.g. ?empty=161b22  or  ?empty=transparent
 *
 * Date Range (mutually exclusive with ?weeks):
 *   ?weeks=53                Number of weeks to show (1–60). Default: 53
 *   ?from=YYYY-MM-DD         Start date (aligns to Monday of that week)
 *   ?to=YYYY-MM-DD           End date   (aligns to Monday of that week). Default: today
 *
 * -----------------------------------------------------------------------------
 * EXAMPLE URLs
 * -----------------------------------------------------------------------------
 *   http://127.0.0.1:1337/workout-heatmap.php
 *   http://127.0.0.1:1337/workout-heatmap.php?style=catppuccin&weeks=26
 *   http://127.0.0.1:1337/workout-heatmap.php?style=halloween&from=2026-01-01&to=2026-04-20
 *   http://127.0.0.1:1337/workout-heatmap.php?style=dark&bg=transparent&empty=transparent
 *
 * -----------------------------------------------------------------------------
 * CLICKHOUSE CONNECTION (environment variables)
 * -----------------------------------------------------------------------------
 *   CLICKHOUSE_URL  Full URL incl. credentials, e.g. http://user:pass@host:8123/?database=mydb
 *                   If set, the individual vars below are ignored.
 *   CH_HOST         Hostname / IP  (required if CLICKHOUSE_URL not set)
 *   CH_PORT         HTTP port.     Default: 8123
 *   CH_DB           Database name. Default: default
 *   CH_USER         Username
 *   CH_PASS         Password.      Default: (empty)
 *   CH_PROTOCOL     http or https. Default: http
 */

header('Content-Type: image/png');
header('Cache-Control: public, max-age=300');
header('Expires: '.gmdate('D, d M Y H:i:s', time() + 300).' GMT');

/* --------------------------- Utilities --------------------------- */
function error_image($msg, $w = 680, $h = 42) {
    $im = imagecreatetruecolor($w, $h);
    imagealphablending($im, true); imagesavealpha($im, true);
    $bg = imagecolorallocatealpha($im, 0, 0, 0, 127); imagefill($im, 0, 0, $bg);
    $red = imagecolorallocate($im, 255, 64, 64);
    imagestring($im, 3, 6, 14, 'Error: '.$msg, $red);
    imagepng($im); imagedestroy($im); exit;
}
function startOfWeekMonday(DateTimeImmutable $dt) {
    $dow = (int)$dt->format('N'); // 1..7 (Mon..Sun)
    $delta = $dow - 1;
    return $dt->setTime(0,0,0)->sub(new DateInterval('P'.$delta.'D'));
}

/**
 * Parse a ?bg= / ?empty= query param.
 * Returns null for 'transparent', a 6-char lowercase hex string for valid hex,
 * or false if the value is absent or invalid (caller should not override).
 */
function parse_hex_param($param) {
    if ($param === null)          return false;
    if ($param === 'transparent') return null;
    if (preg_match('/^#?([0-9a-f]{6}|[0-9a-f]{3})$/', $param, $m)) {
        $hex = $m[1];
        if (strlen($hex) === 3) {
            $hex = $hex[0].$hex[0].$hex[1].$hex[1].$hex[2].$hex[2];
        }
        return $hex;
    }
    return false;
}

/* --------------------------- Theme --------------------------- */
class Theme {
    public $name;
    public $levelHex = array(); // 1..4
    public $borderHex;
    public $emptyHex;  // fill for past days with no data
    public $labelHex;
    public $bgHex = null; // canvas background (null = transparent)

    // Allocated GD colors (filled later for a given image)
    public $level = array(); // 1..4
    public $border;
    public $empty;
    public $label;
    public $bg = null;

    public static function make($style) {
        $s = strtolower((string)$style);
        if ($s === 'light')          return self::light();
        if ($s === 'material_dark')  return self::materialDark();
        if ($s === 'halloween')      return self::halloween();
        if ($s === 'dracula')        return self::dracula();
        if ($s === 'solarized')      return self::solarized();
        if ($s === 'nord')           return self::nord();
        if ($s === 'monokai')        return self::monokai();
        if ($s === 'gruvbox')        return self::gruvbox();
        if ($s === 'one_dark')       return self::oneDark();
        if ($s === 'catppuccin')     return self::catppuccin();
        if ($s === 'tokyo_night')    return self::tokyoNight();
        return self::dark(); // default
    }

    public static function dark() {
        $t = new self();
        $t->name = 'dark';
        // GitHub Dark greens: 0e4429, 006d32, 26a641, 39d353
        $t->levelHex = array(
            1 => '0e4429',
            2 => '006d32',
            3 => '26a641',
            4 => '39d353',
        );
        $t->borderHex = '30363d';
        $t->emptyHex  = '161b22';
        $t->labelHex  = '8b949e';
        $t->bgHex     = '0d1117';
        return $t;
    }

    public static function light() {
        $t = new self();
        $t->name = 'light';
        // GitHub Light greens: 9be9a8, 40c463, 30a14e, 216e39
        $t->levelHex = array(
            1 => '9be9a8',
            2 => '40c463',
            3 => '30a14e',
            4 => '216e39',
        );
        $t->borderHex = 'd0d7de';
        $t->emptyHex  = 'ebedf0';
        $t->labelHex  = '57606a';
        $t->bgHex     = 'ffffff';
        return $t;
    }

    public static function materialDark() {
        $t = new self();
        $t->name = 'material_dark';
        // Material Green ramp
        $t->levelHex = array(
            1 => '1b5e20', // Green 900
            2 => '2e7d32', // Green 800/700
            3 => '43a047', // Green 600
            4 => '66bb6a', // Green 400
        );
        $t->borderHex = '37474f'; // Blue Grey 800
        $t->emptyHex  = '263238'; // Blue Grey 900
        $t->labelHex  = 'b0bec5'; // Blue Grey 200
        $t->bgHex     = '121212';
        return $t;
    }

    public static function halloween() {
        $t = new self();
        $t->name = 'halloween';
        // Classic GitHub Halloween orange ramp
        $t->levelHex = array(
            1 => '631c03',
            2 => 'bd561d',
            3 => 'fa7a18',
            4 => 'fddf68',
        );
        $t->borderHex = '1a1a1a';
        $t->emptyHex  = '1e1600';
        $t->labelHex  = '8b949e';
        $t->bgHex     = '0d1117';
        return $t;
    }

    public static function dracula() {
        $t = new self();
        $t->name = 'dracula';
        // Dracula theme purples
        $t->levelHex = array(
            1 => '44475a',
            2 => '6272a4',
            3 => 'bd93f9',
            4 => 'ff79c6',
        );
        $t->borderHex = '282a36';
        $t->emptyHex  = '383a4a';
        $t->labelHex  = '6272a4';
        $t->bgHex     = '282a36';
        return $t;
    }

    public static function solarized() {
        $t = new self();
        $t->name = 'solarized';
        // Solarized Dark cyan/blue ramp
        $t->levelHex = array(
            1 => '003f4d',
            2 => '006f73',
            3 => '2aa198',
            4 => '93a1a1',
        );
        $t->borderHex = '073642';
        $t->emptyHex  = '073642'; // base02
        $t->labelHex  = '657b83';
        $t->bgHex     = '002b36'; // base03
        return $t;
    }

    public static function nord() {
        $t = new self();
        $t->name = 'nord';
        // Nord polar night -> frost
        $t->levelHex = array(
            1 => '3b4252',
            2 => '5e81ac',
            3 => '81a1c1',
            4 => '88c0d0',
        );
        $t->borderHex = '2e3440';
        $t->emptyHex  = '3b4252'; // nord1
        $t->labelHex  = '616e88';
        $t->bgHex     = '2e3440'; // nord0
        return $t;
    }

    public static function monokai() {
        $t = new self();
        $t->name = 'monokai';
        // Monokai yellow/green
        $t->levelHex = array(
            1 => '3d3d00',
            2 => '75715e',
            3 => 'a6e22e',
            4 => 'e6db74',
        );
        $t->borderHex = '272822';
        $t->emptyHex  = '3e3d32';
        $t->labelHex  = '75715e';
        $t->bgHex     = '272822';
        return $t;
    }

    public static function gruvbox() {
        $t = new self();
        $t->name = 'gruvbox';
        // Gruvbox Dark warm yellow/orange ramp
        $t->levelHex = array(
            1 => '3c3836', // bg2
            2 => 'd65d0e', // orange
            3 => 'd79921', // yellow
            4 => 'fabd2f', // bright yellow
        );
        $t->borderHex = '282828'; // bg
        $t->emptyHex  = '32302f'; // bg1
        $t->labelHex  = 'a89984'; // fg4
        $t->bgHex     = '282828';
        return $t;
    }

    public static function oneDark() {
        $t = new self();
        $t->name = 'one_dark';
        // Atom One Dark green ramp
        $t->levelHex = array(
            1 => '1e4a1e',
            2 => '2d6a2d',
            3 => '98c379', // One Dark green
            4 => 'b5e890',
        );
        $t->borderHex = '282c34'; // One Dark bg
        $t->emptyHex  = '2c313c';
        $t->labelHex  = '5c6370'; // One Dark comment
        $t->bgHex     = '282c34';
        return $t;
    }

    public static function catppuccin() {
        $t = new self();
        $t->name = 'catppuccin';
        // Catppuccin Mocha green ramp
        $t->levelHex = array(
            1 => '1e3a2f',
            2 => '2d5a45',
            3 => '40a02b', // Catppuccin green
            4 => 'a6e3a1', // Catppuccin bright green
        );
        $t->borderHex = '1e1e2e'; // Mocha base
        $t->emptyHex  = '313244'; // Mocha surface0
        $t->labelHex  = '6c7086'; // Mocha overlay1
        $t->bgHex     = '1e1e2e';
        return $t;
    }

    public static function tokyoNight() {
        $t = new self();
        $t->name = 'tokyo_night';
        // Tokyo Night blue/cyan ramp
        $t->levelHex = array(
            1 => '1a2035',
            2 => '1d3557',
            3 => '2ac3de', // Tokyo Night cyan
            4 => '7dcfff', // Tokyo Night bright cyan
        );
        $t->borderHex = '1a1b26'; // Tokyo Night bg
        $t->emptyHex  = '1f2335'; // Tokyo Night night
        $t->labelHex  = '565f89'; // Tokyo Night comment
        $t->bgHex     = '1a1b26';
        return $t;
    }

    private static function alloc_hex($im, $hex, $alpha = 0) {
        $hex = ltrim($hex, '#');
        $r = hexdec(substr($hex, 0, 2));
        $g = hexdec(substr($hex, 2, 2));
        $b = hexdec(substr($hex, 4, 2));
        return imagecolorallocatealpha($im, $r, $g, $b, $alpha);
    }

    public function allocateFor($im) {
        foreach ($this->levelHex as $k => $hex) {
            $this->level[$k] = self::alloc_hex($im, $hex, 0);
        }
        $this->border = self::alloc_hex($im, $this->borderHex, 0);
        $this->empty  = $this->emptyHex !== null ? self::alloc_hex($im, $this->emptyHex, 0) : null;
        $this->label  = self::alloc_hex($im, $this->labelHex,  0);
        $this->bg     = $this->bgHex !== null ? self::alloc_hex($im, $this->bgHex, 0) : null;
    }
}

/* --------------------------- Layout --------------------------- */
class Layout {
    public $cell = 11;   // px
    public $gap  = 2;    // px
    public $rad  = 1;    // px corner radius

    public $leftPad   = 42;
    public $topPad    = 28;
    public $rightPad  = 18;
    public $bottomPad = 28;
}

/* --------------------------- ClickHouse --------------------------- */
class ClickHouseClient {
    private $url;
    private $user;
    private $pass;

    public function __construct() {
        $envUrl = getenv('CLICKHOUSE_URL');
        if ($envUrl) {
            $this->url = $envUrl;
        } else {
            $host = getenv('CH_HOST');
            if (!$host) error_image('CH_HOST not set');
            $port = getenv('CH_PORT') ?: '8123';
            $db   = getenv('CH_DB') ?: 'default';
            $proto= getenv('CH_PROTOCOL') ?: 'http';
            $this->url = $proto.'://'.$host.':'.$port.'/?database='.rawurlencode($db).'&default_format=JSONCompact';
        }
        $this->user = getenv('CH_USER');
        $this->pass = getenv('CH_PASS') ?: '';
    }

    public function fetchPerDay($fromSql, $toSql, $tzName = 'Europe/Berlin') {
        $sql = "
        WITH '" . $tzName . "' AS tz
        SELECT
          toDate(toTimeZone(start_date, tz)) AS day,
          sumIf(weight*reps, weight>0)       AS tonnage
        FROM workouts.workout_sets
        WHERE start_date >= parseDateTimeBestEffort('$fromSql')
          AND start_date <  parseDateTimeBestEffort('$toSql')
        GROUP BY day
        ORDER BY day
        ";

        $ch = curl_init($this->url);
        if ($ch === false) error_image('curl_init failed');
        curl_setopt($ch, CURLOPT_POST, true);
        curl_setopt($ch, CURLOPT_POSTFIELDS, $sql);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, 3);
        curl_setopt($ch, CURLOPT_TIMEOUT, 8);

        if ($this->user !== false && $this->user !== null) {
            curl_setopt($ch, CURLOPT_USERPWD, $this->user.':'.$this->pass);
        }

        $resp = curl_exec($ch);
        $errno = curl_errno($ch);
        $err   = curl_error($ch);
        $code  = curl_getinfo($ch, CURLINFO_RESPONSE_CODE);
        curl_close($ch);

        if ($errno)  error_image($err);
        if ($code < 200 || $code >= 300) error_image("HTTP $code");

        $json = json_decode($resp, true);
        if (!is_array($json) || !isset($json['data'])) error_image('decode failure');

        $perDay = array();
        foreach ($json['data'] as $row) {
            $perDay[$row[0]] = (float)$row[1];
        }
        return $perDay;
    }
}

/* --------------------------- Renderer --------------------------- */
class HeatmapRenderer {
    private $theme;
    private $layout;
    private $tz;

    public function __construct(Theme $theme, Layout $layout, DateTimeZone $tz) {
        $this->theme  = $theme;
        $this->layout = $layout;
        $this->tz     = $tz;
    }

    private static function drawRoundedRect($im, $x, $y, $w, $h, $r, $fill, $border) {
        $r = max(0, min($r, (int)floor(min($w, $h) / 2)));

        // fill
        imagefilledrectangle($im, $x + $r, $y,           $x + $w - 1 - $r, $y + $h - 1,      $fill);
        imagefilledrectangle($im, $x,       $y + $r,     $x + $w - 1,      $y + $h - 1 - $r, $fill);
        if ($r > 0) {
            imagefilledellipse($im, $x + $r,             $y + $r,             $r * 2, $r * 2, $fill);
            imagefilledellipse($im, $x + $w - 1 - $r,    $y + $r,             $r * 2, $r * 2, $fill);
            imagefilledellipse($im, $x + $w - 1 - $r,    $y + $h - 1 - $r,    $r * 2, $r * 2, $fill);
            imagefilledellipse($im, $x + $r,             $y + $h - 1 - $r,    $r * 2, $r * 2, $fill);
        }

        // border
        imagesetthickness($im, 1);
        if ($r > 0) {
            imagearc($im, $x + $r,             $y + $r,             $r * 2, $r * 2, 180, 270, $border);
            imagearc($im, $x + $w - 1 - $r,    $y + $r,             $r * 2, $r * 2, 270,   0, $border);
            imagearc($im, $x + $w - 1 - $r,    $y + $h - 1 - $r,    $r * 2, $r * 2,   0,  90, $border);
            imagearc($im, $x + $r,             $y + $h - 1 - $r,    $r * 2, $r * 2,  90, 180, $border);
            imageline($im, $x + $r,            $y,                  $x + $w - 1 - $r, $y,               $border);
            imageline($im, $x + $w - 1,        $y + $r,             $x + $w - 1,     $y + $h - 1 - $r, $border);
            imageline($im, $x + $r,            $y + $h - 1,         $x + $w - 1 - $r, $y + $h - 1,     $border);
            imageline($im, $x,                 $y + $r,             $x,               $y + $h - 1 - $r, $border);
        } else {
            imagerectangle($im, $x, $y, $x + $w - 1, $y + $h - 1, $border);
        }
    }

    public function render($perDay, DateTimeImmutable $w_from, DateTimeImmutable $w_to) {
        $weeksCount = (int)round(($w_to->getTimestamp() - $w_from->getTimestamp()) / (7*24*3600)) + 1;

        $cell = $this->layout->cell; $gap = $this->layout->gap; $rad = $this->layout->rad;
        $left = $this->layout->leftPad; $top = $this->layout->topPad;
        $right= $this->layout->rightPad; $bottom = $this->layout->bottomPad;

        $width  = $left + $weeksCount * ($cell + $gap) + $right;
        $height = $top  + 7 * ($cell + $gap) + $bottom;

        $im = imagecreatetruecolor($width, $height);
        imagesavealpha($im, true);
        imagealphablending($im, false);
        $transparent = imagecolorallocatealpha($im, 0, 0, 0, 127);
        imagefill($im, 0, 0, $transparent);
        imagealphablending($im, true);

        // allocate theme colors for this image
        $this->theme->allocateFor($im);

        // fill canvas with theme background
        if ($this->theme->bg !== null) {
            imagefilledrectangle($im, 0, 0, $width - 1, $height - 1, $this->theme->bg);
        }

        // robust max (95th percentile)
        $vals = array();
        foreach ($perDay as $v) { if ($v > 0) $vals[] = $v; }
        sort($vals);
        $maxVal = 1.0;
        if (count($vals) > 0) {
            $idx = (int)floor(0.95 * (count($vals) - 1));
            $maxVal = max(1.0, $vals[$idx]);
        }

        // month labels
        $monthNames = array('Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec');
        for ($wi = 0; $wi < $weeksCount; $wi++) {
            $weekStart = $w_from->add(new DateInterval('P'.($wi*7).'D'));
            $prevWeek  = ($wi === 0) ? $weekStart : $w_from->add(new DateInterval('P'.(($wi-1)*7).'D'));
            if ($wi === 0 || $weekStart->format('m') !== $prevWeek->format('m')) {
                $mn = $monthNames[(int)$weekStart->format('n') - 1];
                $x = $left + $wi * ($cell + $gap);
                imagestring($im, 3, $x, 14, $mn, $this->theme->label);
            }
        }

        // weekday labels
        $weekdayLabels = array(1 => 'Tue', 3 => 'Thu', 5 => 'Sat');
        for ($dow = 0; $dow < 7; $dow++) {
            if (isset($weekdayLabels[$dow])) {
                $y = $top + $dow * ($cell + $gap) -1;
                imagestring($im, 2, 12, $y, $weekdayLabels[$dow], $this->theme->label);
            }
        }

        // grid
        $today = (new DateTimeImmutable('today', $this->tz))->format('Y-m-d');
        for ($wi = 0; $wi < $weeksCount; $wi++) {
            $weekStart = $w_from->add(new DateInterval('P'.($wi*7).'D'));
            for ($dow = 0; $dow < 7; $dow++) {
                $day = $weekStart->add(new DateInterval('P'.$dow.'D'))->format('Y-m-d');

                // skip future days
                if ($day > $today) continue;

                $val = isset($perDay[$day]) ? $perDay[$day] : 0.0;

                $level = 0;
                if ($val > 0) {
                    $ratio = $val / $maxVal; // 0..>=1
                    $level = max(1, min(4, (int)ceil($ratio * 4)));
                }

                $x = $left + $wi * ($cell + $gap);
                $y = $top  + $dow * ($cell + $gap);

                $fill = ($level === 0) ? ($this->theme->empty ?? $transparent) : $this->theme->level[$level];
                self::drawRoundedRect($im, $x, $y, $cell, $cell, $rad, $fill, $this->theme->border);
            }
        }

        // legend (bottom right)
        $legendY = $height - 18;
        $legendTextLess = 'Less'; $legendTextMore = 'More';
        $lx = $width - 180; if ($lx < $left) $lx = $left;

        imagestring($im, 2, $lx, $legendY, $legendTextLess, $this->theme->label);
        $bx = $lx + 28;
        for ($i = 1; $i <= 4; $i++) {
            self::drawRoundedRect($im, $bx, $legendY-1, 10, 10, 2, $this->theme->level[$i], $this->theme->border);
            $bx += 13;
        }
        imagestring($im, 2, $bx + 4, $legendY, $legendTextMore, $this->theme->label);

        imagepng($im);
        imagedestroy($im);
    }
}

/* --------------------------- App --------------------------- */

// Inputs
$tz        = new DateTimeZone('Europe/Berlin');
$styleName = isset($_GET['style']) ? (string)$_GET['style'] : 'dark';
$weeks     = isset($_GET['weeks']) ? max(1, min(60, (int)$_GET['weeks'])) : 53;
$toParam   = isset($_GET['to'])   ? $_GET['to']   : null;
$fromParam = isset($_GET['from']) ? $_GET['from'] : null;
$bgParam    = isset($_GET['bg'])    ? strtolower(trim($_GET['bg']))    : null;
$emptyParam = isset($_GET['empty']) ? strtolower(trim($_GET['empty'])) : null;

try {
    $toDate   = $toParam ? new DateTimeImmutable($toParam, $tz) : new DateTimeImmutable('today', $tz);
} catch (Exception $e) {
    error_image('Invalid "to" date: '.$toParam);
}
try {
    $fromDate = $fromParam ? new DateTimeImmutable($fromParam, $tz) : $toDate->sub(new DateInterval('P'.($weeks*7-1).'D'));
} catch (Exception $e) {
    error_image('Invalid "from" date: '.$fromParam);
}

// Align to whole weeks (Mon..Sun)
$w_from = startOfWeekMonday($fromDate);
$w_to   = startOfWeekMonday($toDate);

// Fetch data
$fromSql = $w_from->format('Y-m-d 00:00:00');
$toSql   = $w_to->add(new DateInterval('P7D'))->format('Y-m-d 00:00:00');

$client  = new ClickHouseClient();
$perDay  = $client->fetchPerDay($fromSql, $toSql, $tz->getName());

// Render
$theme   = Theme::make($styleName);
// Apply ?bg= and ?empty= overrides
$bg = parse_hex_param($bgParam);
if ($bg !== false) $theme->bgHex = $bg;

$empty = parse_hex_param($emptyParam);
if ($empty !== false) $theme->emptyHex = $empty;

$layout  = new Layout();
$renderer= new HeatmapRenderer($theme, $layout, $tz);
$renderer->render($perDay, $w_from, $w_to);