;;; compmd-ts-mode.el --- Major mode for CompMD, Markdown with components -*- lexical-binding: t; -*-

;; Copyright (C) 2026 The CompMD authors

;; Author: The CompMD authors
;; URL: https://github.com/comp-markdown/compmd-ts-mode
;; Version: 0.11.0
;; Package-Requires: ((emacs "30.1"))
;; Keywords: languages, text
;; SPDX-License-Identifier: MIT

;; This file is not part of GNU Emacs.

;;; Commentary:

;; CompMD (https://compmd.dev) is a Markdown flavor with components, containers, slots and layout.  This mode edits
;; `.comp.md' files with Emacs's built-in tree-sitter and CompMD's grammars (`compmd', `compmd-inline' and `cel', from
;; comp-markdown/compmd-grammars), and its language server, `compmd-lsp', through Eglot:
;;
;; - highlighting, of Markdown and CompMD's components, params, interpolations and expressions (CEL);
;; - Imenu and outlines of the headings and the containers;
;; - the server's diagnostics, completion, hover and code actions with `M-x eglot';
;; - `compmd-ts-preview' (C-c C-p), the document's preview in the browser, as the server serves it.
;;
;; `M-x treesit-install-language-grammar' installs the three grammars.  When `compmd-lsp' isn't found, Eglot asks
;; whether to download it: the release's binary for this platform, its SHA-256 sum checked.

;;; Code:

(require 'treesit)
(require 'url)

(declare-function eglot-current-server "eglot")
(declare-function eglot-execute "eglot")
(declare-function eglot-path-to-uri "eglot")
(defvar eglot-server-programs)

(defgroup compmd-ts nil
  "CompMD, Markdown with components."
  :group 'languages
  :prefix "compmd-ts-")

(defcustom compmd-ts-lsp-version "0.11.0"
  "The release of `compmd-lsp' that `compmd-ts-lsp-install' downloads."
  :type 'string)

(defcustom compmd-ts-lsp-install-directory (locate-user-emacs-file "compmd-lsp")
  "Where `compmd-ts-lsp-install' puts the language server."
  :type 'directory)

(defconst compmd-ts--releases "https://github.com/comp-markdown/compmd/releases/download"
  "Where the language server's releases are, each under its tag.")

(defconst compmd-ts--grammars "https://github.com/comp-markdown/compmd-grammars"
  "The repository of CompMD's tree-sitter grammars.")

;; The recipes' positional form (URL REVISION SOURCE-DIR): Emacs 30 reads no keywords in them.
(dolist (grammar '((compmd "tree-sitter/compmd/src")
                   (compmd-inline "tree-sitter/compmd-inline/src")
                   (cel "tree-sitter/cel/src")))
  (add-to-list 'treesit-language-source-alist
               `(,(car grammar) ,compmd-ts--grammars nil ,(cadr grammar))
               t))

;;; Faces

(defface compmd-ts-heading '((t :inherit (bold font-lock-function-name-face)))
  "A heading.")

(defface compmd-ts-code '((t :inherit font-lock-string-face))
  "A code span or a code block.")

;;; Highlighting

(defvar compmd-ts--font-lock-settings
  (treesit-font-lock-rules
   :language 'compmd
   :feature 'heading
   '([(atx_heading) (setext_heading)] @compmd-ts-heading)

   :language 'compmd
   :feature 'code
   '([(fenced_code_block) (indented_code_block)] @compmd-ts-code
     (info_string (language) @font-lock-type-face))

   :language 'compmd
   :feature 'markdown
   '([(list_marker_plus) (list_marker_minus) (list_marker_star) (list_marker_dot) (list_marker_parenthesis)
      (thematic_break) (block_quote_marker)]
     @font-lock-delimiter-face
     (link_destination) @link
     (backslash_escape) @font-lock-escape-face)

   :language 'compmd
   :feature 'frontmatter
   '([(minus_metadata) (plus_metadata)] @font-lock-comment-face
     (frontmatter_key) @font-lock-preprocessor-face)

   :language 'compmd
   :feature 'component
   :override t
   '((component_marker) @font-lock-delimiter-face
     (container_closer "/" @font-lock-delimiter-face)
     (component_slot "#" @font-lock-delimiter-face)
     (param_continuation "+" @font-lock-delimiter-face)
     (component_name) @font-lock-type-face
     (slot_name) @font-lock-builtin-face
     (definition_separator) @font-lock-delimiter-face)

   :language 'compmd
   :feature 'param
   :override t
   '((param_key (param_name) @font-lock-property-name-face)
     (breakpoint) @font-lock-builtin-face
     (param ["." "#" (class_name) (id_name)] @font-lock-property-name-face)
     (param "=" @font-lock-operator-face)
     (string) @font-lock-string-face
     (string_escape) @font-lock-escape-face
     (bare_value) @font-lock-constant-face
     (param_list ["{" "}"] @font-lock-bracket-face)
     (fallback ["[" "]"] @font-lock-bracket-face)
     (expression ["(" ")"] @font-lock-delimiter-face))

   ;; A definition's name, `card' of `component=card', is a component's name, not a bare value: a rule of its own,
   ;; after the params', so it wins over theirs.
   :language 'compmd
   :feature 'definition
   :override t
   '(((info_string (param key: (param_key name: (param_name) @_key) value: (bare_value) @font-lock-type-face))
      (:equal @_key "component")))

   :language 'compmd-inline
   :feature 'markdown
   '((code_span) @compmd-ts-code
     (emphasis) @italic
     (strong_emphasis) @bold
     (strikethrough) @font-lock-comment-face
     [(link_text) (image_description)] @font-lock-doc-face
     [(link_destination) (uri_autolink) (email_autolink)] @link
     [(backslash_escape) (hard_line_break)] @font-lock-escape-face)

   :language 'compmd-inline
   :feature 'component
   :override t
   '((inline_component (component_marker) @font-lock-delimiter-face)
     (component_name) @font-lock-type-face
     (interpolation ["{{" "}}"] @font-lock-delimiter-face)
     (interpolation_path) @font-lock-variable-use-face
     (wiki_link (link_destination) @link))

   :language 'compmd-inline
   :feature 'param
   :override t
   '((param_key (param_name) @font-lock-property-name-face)
     (breakpoint) @font-lock-builtin-face
     (param ["." "#" (class_name) (id_name)] @font-lock-property-name-face)
     (param "=" @font-lock-operator-face)
     (string) @font-lock-string-face
     (string_escape) @font-lock-escape-face
     (bare_value) @font-lock-constant-face
     (param_list ["{" "}"] @font-lock-bracket-face)
     (fallback ["[" "]"] @font-lock-bracket-face)
     (expression ["(" ")"] @font-lock-delimiter-face))

   :language 'cel
   :feature 'expression
   :override t
   '((identifier) @font-lock-variable-use-face
     ((identifier) @font-lock-builtin-face (:equal @font-lock-builtin-face "md"))
     (member_expression member: (identifier) @font-lock-property-use-face)
     (call_expression function: (identifier) @font-lock-function-call-face)
     (call_expression function: (member_expression member: (identifier) @font-lock-function-call-face))
     (number) @font-lock-number-face
     (string) @font-lock-string-face
     (bytes) @font-lock-string-face
     [(true) (false) (null)] @font-lock-constant-face
     (comment) @font-lock-comment-face
     ["||" "&&" "==" "!=" "<" "<=" ">" ">=" "+" "-" "*" "/" "%" "!" "?" ":" "in"] @font-lock-operator-face))
  "How `compmd-ts-mode' highlights, by the grammars' nodes.
The captures of the grammars' own `highlights.scm', as Emacs's faces.")

(defvar compmd-ts--range-settings
  (treesit-range-rules
   :embed 'compmd-inline
   :host 'compmd
   :local t
   '([(inline) (pipe_table_cell)] @compmd-inline)

   :embed 'cel
   :host 'compmd
   :local t
   '((expression_code) @cel)

   :embed 'cel
   :host 'compmd-inline
   :local t
   '((expression_code) @cel))
  "The grammars' injections: each block's inlines, and each expression's CEL.")

;;; Navigation

(defun compmd-ts--container-name (node)
  "The name of the container NODE, for Imenu and outlines."
  (when-let* ((name (treesit-search-subtree node "component_name" nil nil 2)))
    (concat "::: " (treesit-node-text name t))))

(defun compmd-ts--heading-name (node)
  "The text of the heading NODE, for Imenu and outlines."
  (string-trim (replace-regexp-in-string "\\`#+\\|\n.*" "" (treesit-node-text node t))))

(defun compmd-ts--outline-predicate (node)
  "Whether NODE starts an outline entry: a heading or a container."
  (member (treesit-node-type node) '("atx_heading" "setext_heading" "component_container")))

;;; The language server

(defun compmd-ts--lsp-target ()
  "The release target of this platform, as the release names its binaries."
  (let ((arch (if (string-match-p "\\`\\(aarch64\\|arm64\\)" system-configuration) "aarch64" "x86_64")))
    (pcase system-type
      ('darwin (concat arch "-apple-darwin"))
      ('windows-nt (concat arch "-pc-windows-msvc"))
      (_ (concat arch "-unknown-linux-musl")))))

(defun compmd-ts--lsp-binary-name ()
  "The release's binary for this platform."
  (format "compmd-lsp-%s-%s%s" compmd-ts-lsp-version (compmd-ts--lsp-target)
          (if (eq system-type 'windows-nt) ".exe" "")))

(defun compmd-ts--lsp-installed ()
  "The server `compmd-ts-lsp-install' put in place, if it did."
  (let ((path (expand-file-name (if (eq system-type 'windows-nt) "compmd-lsp.exe" "compmd-lsp")
                                compmd-ts-lsp-install-directory)))
    (and (file-executable-p path) path)))

(defun compmd-ts--sum-of (file)
  "The SHA-256 sum of FILE's bytes."
  (with-temp-buffer
    (set-buffer-multibyte nil)
    (insert-file-contents-literally file)
    (secure-hash 'sha256 (current-buffer))))

(defun compmd-ts--expected-sum (sums name)
  "The sum SUMS, a `SHA256SUMS' file's text, gives the file NAME."
  (catch 'found
    (dolist (line (split-string sums "\n" t))
      (when (string-match "\\`\\([0-9a-f]\\{64\\}\\) [ *]?\\(.+\\)\\'" line)
        (when (equal (match-string 2 line) name)
          (throw 'found (match-string 1 line)))))
    nil))

;;;###autoload
(defun compmd-ts-lsp-install ()
  "Download `compmd-lsp' for this platform and put it in place.
The binary is `compmd-ts-lsp-version' release's, its SHA-256 sum
checked against the release's SHA256SUMS.  Return its path.  Signal
an error when the sum doesn't match, keeping nothing."
  (interactive)
  (let* ((name (compmd-ts--lsp-binary-name))
         (base (format "%s/v%s/" compmd-ts--releases compmd-ts-lsp-version))
         (target (expand-file-name (if (eq system-type 'windows-nt) "compmd-lsp.exe" "compmd-lsp")
                                   compmd-ts-lsp-install-directory))
         (download (make-temp-file "compmd-lsp-"))
         (sums (with-temp-buffer
                 (url-insert-file-contents (concat base "SHA256SUMS"))
                 (buffer-string))))
    (unwind-protect
        (let ((expected (or (compmd-ts--expected-sum sums name)
                            (error "The release %s lists no %s" compmd-ts-lsp-version name))))
          (url-copy-file (concat base name) download t)
          (unless (equal (compmd-ts--sum-of download) expected)
            (error "%s doesn't match its sum: not installed" name))
          (make-directory compmd-ts-lsp-install-directory t)
          (copy-file download target t)
          (set-file-modes target #o755)
          (message "compmd-lsp %s installed in %s" compmd-ts-lsp-version target)
          target)
      (delete-file download))))

(defun compmd-ts--lsp-program (&optional interactive)
  "How Eglot starts the language server: `compmd-lsp' on the path or installed.
When there is none and INTERACTIVE, ask whether to download it."
  (let ((found (or (executable-find "compmd-lsp") (compmd-ts--lsp-installed))))
    (when (and (not found) interactive
               (yes-or-no-p (format "The language server compmd-lsp isn't installed: download its release %s? "
                                    compmd-ts-lsp-version)))
      (setq found (compmd-ts-lsp-install)))
    (list (or found "compmd-lsp") "--stdio")))

;; Eglot and project.el learn of CompMD when they load, without the mode loading them.
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(compmd-ts-mode . compmd-ts--lsp-program)))

;; A CompMD project is the folder of its configuration, which the server reads, for Eglot as for `project-find-file'.
(defvar project-vc-extra-root-markers)
(with-eval-after-load 'project
  (add-to-list 'project-vc-extra-root-markers "compmd.config.json"))

(defun compmd-ts-preview ()
  "Open the document's preview in the browser.
The language server serves the preview and opens the browser at it."
  (interactive)
  (require 'eglot)
  (let ((server (or (eglot-current-server) (user-error "Start the language server first: M-x eglot"))))
    (eglot-execute server (list :command "compmd.preview"
                                :arguments (vector (eglot-path-to-uri (buffer-file-name)))))))

;;; The mode

(defvar-keymap compmd-ts-mode-map
  :doc "Keymap for `compmd-ts-mode'."
  "C-c C-p" #'compmd-ts-preview)

;;;###autoload
(define-derived-mode compmd-ts-mode text-mode "CompMD"
  "Major mode for CompMD documents, with tree-sitter.

\\{compmd-ts-mode-map}"
  (setq-local comment-start "<!-- ")
  (setq-local comment-end " -->")
  (when (treesit-ready-p 'compmd)
    (treesit-parser-create 'compmd)
    (when (treesit-ready-p 'compmd-inline t)
      (when (treesit-ready-p 'cel t)
        (setq-local treesit-range-settings compmd-ts--range-settings)))
    (setq-local treesit-font-lock-settings compmd-ts--font-lock-settings)
    (setq-local treesit-font-lock-feature-list
                '((heading component definition)
                  (frontmatter param code)
                  (markdown expression)))
    (setq-local treesit-simple-imenu-settings
                `(("Heading" ,(regexp-opt '("atx_heading" "setext_heading")) nil compmd-ts--heading-name)
                  ("Container" "\\`component_container\\'" nil compmd-ts--container-name)))
    (setq-local treesit-outline-predicate #'compmd-ts--outline-predicate)
    (treesit-major-mode-setup)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.comp\\.md\\'" . compmd-ts-mode))

(provide 'compmd-ts-mode)

;;; compmd-ts-mode.el ends here
