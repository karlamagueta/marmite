**WARNING** This project is in its very initial development stage, not all
features are implemented yet, usage API still subjected to change until `1.0.0`


![Logo](/assets/_resized/logo_160x120.png)

# Marmite

[**Mar**kdown **M**akes s**ite**s] is a **very!** simple static site generator.

## How it works

It does **"one"** simple thing only:

- Reads all `.md` files on the directory.
- Using `CommonMark` parse it to `HTML` content.
- Extract metadata from `frontmatter` or `filename`.
- Renders each content to `html` (templates are customizable).
- Outputs the rendered static site to the `output` folder.

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)
![Crates.io Version](https://img.shields.io/crates/v/marmite)

## Installation

Install with cargo

```bash
cargo install marmite
```

<!--
Or download the pre-built binary from the [releases](https://github.com/rochacbruno/marmite/releases) page. 
-->

## Usage

It's simple, really!

```bash
marmite path_to_markdown_files path_to_generated_site
```

> **WARNING** CLI not completely implemented yet, see: [#1](https://github.com/rochacbruno/marmite/issues/1)

```bash
marmite [input_folder] [output_folder] [OPTIONS]

input_folder: A folder containing `.md` files.
  and optionally: marmite.yaml, templates, static, media folders

output_folder: Directory where the static site will be rendered.

OPTIONS:
  --serve    Build and Serve with embedded http server
  --config   Optional path to a config file
  --name     Site name
  --tagline  Text shown in the site header
  --url      Url where site will be deployed (used when generating feeds)
```

### Build a site from markdown content

Put some markdown in a folder.

```console
$ mkdir myblog
$ echo "# page ..." > myblog/my-page.md
$ echo "# post ..." > myblog/2024-01-31-my-first-post.md
```

Or use your favorite markdown editor.

Then, build:

```console
$ marmite myblog site

building index.html
building pages.html
building tags.html
building archive.html
building feed.rss
building feed.json
building 404.html

building my-page.html
building my-first-post.html

Site generated at site/ 
```

### Result

Site is generated from embedded templates that are purposely very simple!

- **Front page** lists all blog posts (markdown containing `date` attribute)
- **Menu** is shown with links to `pages`, `tags`, `archive` (menu is customizable)
- **Feeds** are generated as RSS and JSON formats
- **Content** pages are generated from every `.md` file

Deploy `site/` folder to your favorite webserver.

Open `index.html` on your web-browser or run `marmite myblog site --serve` to run
the embedded webserver.

## Customization

Marmite allows customization of the website using custom `templates` that
are written using `Tera` template language (similar to Jinja and Twig).

Site metadata can be customized on `marmite.yaml`.

### Folder structure

```plain
myblog
├── marmite.yaml        # Site configuration
├── content
│   └── *.md            # Site content
├── static
│   └── *.css|js|ttf    # Static files (CSS, JS, Fonts)
└── templates
    ├── base.html       # Common HTML
    ├── content.html    # Renders page and post
    ├── group.html      # Archive and tags list
    └── list.html       # Renders index, tags, archive
```

### Optional configuration

> All keys are optional, but you probably want to set at least `name`,`tagline`, `url`

`marmite.yaml`
```yaml
name: My Blog
tagline: This blog is awesome
url: https://www.myblog.com
# footer: This is an example site generated with Marmite
# pagination: 10

# list_title: Blog Posts
# tags_title: Tags
# archive_title: Archive

# templates_path: templates
# static_path: static
# media_path: media
# card_image: name of site card image, relative to media or absolute
# logo_image: name of site logo image, relative to media or absolute

# menu: Optional menu items
# data: Custom key:pair values to be exposed to template context.
```

### Content types

- **Post**: If `.md` has a `date` property on filename prefix or `frontmatter` it is considered a post to show in the index.html posts list.
- **Page**: If it does not have a `date` it is considered a page, listed on pages.html and acessible via direct link.

### Metadata

FrontMatter can optionally be specified in `yaml` format.

```yaml
---
date: "2024-01-01"
title: Title
slug: title
tags: comma,separated,tags
tags:
  - yaml
  - list
  - also
  - works
---
# Title of my content

Content Text ...
```

- **date**: If informed, the content is considered a `Post` and shows in index list.
- **title**: If not defined, the first line of the markdown content is used.
- **slug**: If not defined, the filename is used to build the url.
- **tags**: If not defined en empty list of tags will be added.

All fields are optional, if nothing is declared then the content is considered an unlisted `Page`.

### Example content

`content/first-post.md`
````markdown
---
date: "2024-01-01"
title: My First Blog Post
slug: my-first-blog-post
tags: poetry,life
---

# Hello this is my first post

This blog was generated by `Marmite` the simplest static site generator

## Images

![local image](/media/simple.png)

I can also have remote images

![remote image](https://github.com/rochacbruno/marmite/logo.png)


## Code Snippets

```python
de foo():
    return "bar"
```

Everything CommonMark and Github flavoured markdown supports.
````

## editors and deployment

Marmite does not come with an editor but as the content is simply markdown files, **any** text editor will work!

Generated site is pure HTML + CSS so it can be served on **any** static webserver.

Workflow is generally:

- Create a new `content/your-content-title.md`.
- Edit in your preferred **editor** (There are some with good preview support).
- Add the `date` metadata when it is ready to publish.
- commit to your preferred **repository**.
- use your preferred **automation system** to publish it to your preferred **web server**.

Common examples:

- Edit in **Marktext** Editor, configure it to move pasted images to `media`, commit to Github, Add an action to build and Publish as a Github Page.
- Edit directly in the Github Web UI, commit, let CI Action to build and publish.
- Edit in **vim**, Generate the site locally, Publish via FTP.
- Edit in **VsCode**, Commit to Git Repository, Have the CI to build and Publish (GH pages, netlify etc)

> Marmite focus on generating the site from markdown only, the deployment and media management is a separate problem to solve.

## That's all!

**Marmite** is very simple, and limited in functionality, there is no intention to add more features or built-in themes.

If this simplicity does not suit your needs, there are other awesome static site generators.


Here are some that I recommend:

- [Cobalt](https://cobalt-org.github.io/)
- [Zola](https://www.getzola.org/)