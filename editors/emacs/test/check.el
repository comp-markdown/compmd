;;; check.el --- compmd-ts-mode checked in a batch Emacs, against local builds -*- lexical-binding: t; -*-

;;; Commentary:

;; The mode, the three grammars and the language server through Eglot, in an Emacs with no window.
;; `cargo make emacs-check' builds what it needs and runs it:
;;
;;   emacs --batch -l editors/emacs/test/check.el <built grammars' folder> <compmd-lsp>
;;
;; It exits 0 when everything holds, else 1, saying what didn't.

;;; Code:

(require 'eglot)
(require 'imenu)

(defvar compmd-check--failures 0 "How many checks failed.")

(defun compmd-check (holds what)
  "Say whether WHAT HOLDS, counting it if not."
  (princ (format "%s %s\n" (if holds "ok  " "FAIL") what))
  (unless holds (setq compmd-check--failures (1+ compmd-check--failures))))

(defun compmd-check--wait (predicate)
  "Wait up to 20 seconds for PREDICATE to hold; whether it did."
  (let ((until (+ (float-time) 20)) held)
    (while (and (not (setq held (funcall predicate))) (< (float-time) until))
      (accept-process-output nil 0.05))
    held))

(defun compmd-check--faces-at (position)
  "The faces at POSITION, as a list."
  (let ((face (get-text-property position 'face)))
    (if (listp face) face (list face))))

(let* ((here (file-name-directory (directory-file-name (file-name-directory load-file-name))))
       (grammars (expand-file-name (pop command-line-args-left)))
       (server (expand-file-name (pop command-line-args-left)))
       (folder (make-temp-file "compmd-emacs-" t))
       (document (expand-file-name "notes.comp.md" folder)))
  (load (expand-file-name "compmd-ts-mode.el" here) nil t)
  (setq treesit-extra-load-path (list grammars))
  ;; A CompMD project, which the mode tells project.el to find by its configuration.
  (with-temp-file (expand-file-name "compmd.config.json" folder) (insert "{}\n"))
  ;; A component its project doesn't have (F002), an interpolation, an expression and a definition.
  (with-temp-file document
    (insert "---\ntitle: Harbour notes\n---\n\n# {{md.doc.title}}\n\n"
            "::: nowhere {wide=(1 + 2)}\nThe tide is :mark[in].\n::: /nowhere\n\n"
            "```compmd component=tide\nparams:\n  level: number\n---\nThe tide is {{level}} metres.\n```\n"))
  (find-file document)
  (compmd-check (eq major-mode 'compmd-ts-mode) "a .comp.md file is in compmd-ts-mode")
  (compmd-check (treesit-parser-list) "the compmd parser is made")
  (compmd-check (equal (project-root (project-current)) (file-name-as-directory folder)) "the project is the configuration's folder")
  (font-lock-ensure)
  (cl-flet ((face-at (text offset face what)
              (goto-char (point-min))
              (search-forward text)
              (let ((faces (compmd-check--faces-at (+ (match-beginning 0) offset))))
                (compmd-check (memq face faces) (format "%s is %s%s" what face (if (memq face faces) "" (format ", not %S" faces)))))))
    (face-at "nowhere {" 0 'font-lock-type-face "a container's name")
    (face-at "wide=" 0 'font-lock-property-name-face "a param's name")
    (face-at "md.doc.title" 0 'font-lock-variable-use-face "an interpolation's path (compmd-inline, embedded)")
    (face-at "mark[" 0 'font-lock-type-face "an inline component's name (compmd-inline, embedded)")
    (face-at "1 + 2" 0 'font-lock-number-face "an expression's number (cel, embedded)")
    (face-at "tide\n" 0 'font-lock-type-face "a definition's name"))
  (let ((index (imenu--make-index-alist t)))
    (compmd-check (assoc "Container" index) "Imenu lists the containers")
    (compmd-check (assoc "Heading" index) "Imenu lists the headings"))

  (setq-default eglot-workspace-configuration '(:compmd (:preview (:browser "true"))))
  (setq eglot-server-programs `((compmd-ts-mode ,server "--stdio")))
  (eglot 'compmd-ts-mode (project-current) 'eglot-lsp-server (list server "--stdio") "compmd")
  (let ((managed (compmd-check--wait #'eglot-current-server)))
    (compmd-check managed "compmd-lsp starts through Eglot"))
  (compmd-check (compmd-check--wait
                 (lambda ()
                   (seq-some (lambda (diagnostic) (string-search "unknown component `nowhere`" (flymake-diagnostic-text diagnostic)))
                             (flymake-diagnostics))))
                "the unknown component is diagnosed (F002)")
  (when-let* ((server (eglot-current-server)))
    (let* ((uri (eglot-path-to-uri document))
           (origin '(:line 0 :character 0))
           (actions (jsonrpc-request server :textDocument/codeAction
                                     `(:textDocument (:uri ,uri) :range (:start ,origin :end ,origin) :context (:diagnostics [])))))
      (compmd-check (seq-some (lambda (action) (equal (plist-get action :title) "Open preview")) actions)
                    "Open preview is a code action"))
    (compmd-check (condition-case nil (progn (compmd-preview) t) (error nil)) "compmd-preview runs the server's command")
    (let* ((url (jsonrpc-request server :workspace/executeCommand
                                 `(:command "compmd.preview" :arguments [,(eglot-path-to-uri document)])))
           (page (and (stringp url) (string-match-p "\\`http://127\\.0\\.0\\.1:[0-9]+/[0-9a-f]+/\\'" url)
                      (url-retrieve-synchronously url t t 20))))
      (compmd-check (and page (with-current-buffer page (goto-char (point-min)) (looking-at "HTTP/1\\.[01] 200")))
                    "compmd.preview serves the page"))
    (eglot-shutdown server))

  (compmd-check (equal (compmd-lsp--expected-sum "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  compmd-lsp-0.11.0-aarch64-apple-darwin\n"
                                                 "compmd-lsp-0.11.0-aarch64-apple-darwin")
                       "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                "a release binary's sum is read from SHA256SUMS")
  (compmd-check (string-match-p "\\`compmd-lsp-0\\.11\\.0-\\(aarch64\\|x86_64\\)-" (compmd-lsp--binary-name))
                "the release binary for this platform is named as the release names it")

  (if (> compmd-check--failures 0)
      (progn (princ (format "%d failed\n" compmd-check--failures)) (kill-emacs 1))
    (princ "all hold\n")
    (kill-emacs 0)))

;;; check.el ends here
