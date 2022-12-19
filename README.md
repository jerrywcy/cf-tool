<!-- @format -->

# CodeForces Tool

This project is inspired by [cf-tool](https://github.com/xalanq/cf-tool), which has deprecated since change in CodeForces API.

Code Forces tool is a terminal user interface tool for [CodeForces](https://codeforces.com/).

## Features

-   Support Contests, Gym, Groups and acmsguru.
-   Support all programming languages in Codeforces.
-   Submit codes (Copy codes to clipboard and open it in default web browser).
-   Watch submissions' status dynamically.
-   Fetch problems' samples.
-   Compile and test locally.
-   Generate codes from the specified template (including timestamp, author, etc.)
-   List problems' stats of one contest.
-   Use default web browser to open problems' pages, standings' page, etc.
-   Colorful TUI.

## Installation

### Manual Installation

```bash
git clone https://github.com/jerrywcy/cf-tool.git
cd cf-tool
cargo install --path . --bin cf-tui
```

Then enjoy cf-tool!

## Screenshots

![image](https://user-images.githubusercontent.com/34678939/208440257-7548a317-7198-40c8-8bae-e6b150f08c8d.png)

![Pasted image 20221218172037](https://user-images.githubusercontent.com/34678939/208440843-434d02b1-8632-4b4b-9962-eee75b251dc7.png)

![Pasted image 20221218172516](https://user-images.githubusercontent.com/34678939/208440891-5c5a96bb-ea3d-4be5-b3ad-1bcee02bb0c2.png)

![Pasted image 20221218172531](https://user-images.githubusercontent.com/34678939/208440927-352723ca-70f6-4f96-90f1-11ee57af2a46.png)

![Pasted image 20221218172556](https://user-images.githubusercontent.com/34678939/208440953-3f351e0d-6c1f-44dc-bd24-189df0d02aec.png)

![Pasted image 20221218172717](https://user-images.githubusercontent.com/34678939/208440971-02219835-8457-420d-ac91-fa8e9df9fcbe.png)

![Pasted image 20221218172947](https://user-images.githubusercontent.com/34678939/208440995-9efb84ca-a73c-462e-80ae-49daa1e45e8b.png)

![Pasted image 20221218173001](https://user-images.githubusercontent.com/34678939/208441011-701c181a-ed0f-4a5e-ab5d-d1714da0f047.png)

## Usage

-   Run `cf-tui` to enter the main browser view.
-   Run `cf-tui race <CONTEST_ID>` to take part in a give contest.
-   Run `cf-tui config` to configure cf-tool using a configuration guide.

-   Press `q` or `Esc` to exit current view.
-   Press `Tab` or `h` or `Right` to go to the next tab.
-   Press `Shift + Tab` or `l` or `Left` to go to the previous tab.
-   Press `Enter` on contests to enter contest view.
-   Press `Enter` on problems to open the problem in default web browser.
-   Press `p` on problems to parse sample for the current problem.
-   Press `P` on problems tab to parse all samples for the current contest.
-   Press `g` on problems to generate codes according to template for the current problem.
-   Press `o` on problems to open them using the `open_script` configured.
-   Press `t` on problems to test current problems.

## Configuration

### Files

cf-tool will save data in `config_dir`.

Normally, `config_dir` is `~/.config/cf` where `~` is the home directory of current user in your system.

In config_dir there are following files:

-   `config_dir/cf.json` Your configuration file.
-   `config_dir/templates` The folder to store templates.

Configuration file consists of the following parts:

### Login

-   `username`: Your username.
-   `key`: Your API key.
-   `secret`: Your API secret.

API key and secret can be created [here](https://codeforces.com/settings/api).

### Templates

`templates` is an array of templates.

Each template is configured as the following:

```json
{
    \"alias\": The alias of this template.
               This can be anything you want to recoginize this template but must be configured.
    \"lang\":  The language this template uses.
               This can be anything you want to recoginize this template and doesn't follow any restriction.
               For example, this can be configured as either \"cpp\" or \"C++\".
               This must be configured.
    \"path\":  The path to your code for this template. cf-tool will read from this directory when the path specified in templates in configuration file is not an absolute path.
               For example, when path = /home/cf/dcode.cpp , cf-tool will read from /home/cf/dcode.cpp .
                            when path = dcode.cpp, cf-tool will read from config_dir/templates/dcode.cpp;
}
```

You can insert some placeholders into your template code.

When generate a code from the template, cf will replace all placeholders by following rules:

| Placeholder      | Content              |
| :--------------- | :------------------- |
| `<% username %>` | Handle (e.g. xalanq) |
| `<% year %>`     | Year (e.g. 2022)     |
| `<% month %>`    | Month (e.g. 12)      |
| `<% day %>`      | Day (e.g. 19)        |
| `<% hour %>`     | Hour (e.g. 09)       |
| `<% minute %>`   | Minute (e.g. 08)     |
| `<% second %>`   | Second (e.g. 02)     |

For example, the following code:

```cpp
/*
 *   user: <% username %>
 *   time: <% year %>-<% month %>-<% day %> <% hour %>:<% minute %>:<% second %>
 */

#include <bits/stdc++.h>

using namespace std;

int main(){

    return 0;

}
```

will be generated as:

```cpp
/*
 *  user: jerrywcy
 *  time: 2022-12-19 18:01:05
 */

#include <bits/stdc++.h>

using namespace std;

int main(){

    return 0;

}
```

### Commands

cf-tool will run 3 scripts in sequence when you run press `t` on a problem:

-   `before_script` (execute once)
-   `script` (execute the number of samples times)
-   `after_script` (execute once)

You could set `before_script` or `after_script` to empty string, meaning not executing.
You have to run your program in script with standard input/output (no
need to redirect).

You can insert some placeholders in your scripts. When execute a script,
cf will replace all placeholders by following rules:

| Placeholder | Content                                                              |
| :---------- | :------------------------------------------------------------------- |
| <% path %>  | Path to source file (Excluding `<% full %>`, e.g. `/home/jerrywcy/`) |
| <% full %>  | Full name of source file (e.g. `a.cpp`)                              |
| <% file %>  | Name of source file (Excluding suffix, e.g. `a`)                     |

For example, a script that runs C++ can be written as follow:

```json
{
    "before_script": "g++ <% full %> -o <% file %>",
    "script": "./<% file %>" (or "./<% file %>.exe" on Windows system)
}
```

### Directories

-   `home_dir`: The directory that stores all codes and samples generated by cf-tool and the directory to read from when testing or submitting.

The directory looks like the following:

```bash
home_dir
└── Contests
    └── contest_id
        └── problem_index
              ├── problem_index.cpp
              ├── ans1.txt
              ├── ans2.txt
              ├── in1.txt
              └── in2.txt
```
