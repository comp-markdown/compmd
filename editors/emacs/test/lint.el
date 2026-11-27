;;; lint.el --- compmd-ts-mode.el checked as MELPA checks it -*- lexical-binding: t; -*-

;;; Commentary:

;; package-lint, checkdoc and byte-compiling with warnings as errors, in a batch Emacs:
;;
;;   emacs --batch -l test/lint.el <package-user-dir>
;;
;; package-lint is installed from MELPA into <package-user-dir>, a folder of its own, the first time.  It exits 0
;; when the three find nothing, else 1, saying what they found.

;;; Code:

(require 'package)
(require 'checkdoc)

(defvar compmd-lint--problems 0 "How many problems the checks found.")

(defun compmd-lint--found (what)
  "Say WHAT a check found, counting it."
  (princ (format "%s\n" what))
  (setq compmd-lint--problems (1+ compmd-lint--problems)))

(defun compmd-lint--install-package-lint (directory)
  "Make package-lint loadable, installing it from MELPA into DIRECTORY if it isn't there."
  (setq package-user-dir (expand-file-name directory))
  (add-to-list 'package-archives '("melpa" . "https://melpa.org/packages/") t)
  (package-initialize)
  (unless (package-installed-p 'package-lint)
    (package-refresh-contents)
    (package-install 'package-lint))
  (require 'package-lint))

(defconst compmd-lint--accepted-warnings
  '("`with-eval-after-load' is for use in configurations")
  "The package-lint warnings the mode keeps, each for a reason.
`with-eval-after-load': the mode adds its server to Eglot's
`eglot-server-programs' and its configuration to project.el's root
markers when those load, without loading them itself.")

(defun compmd-lint--package-lint (file)
  "Run package-lint on FILE, counting what it finds but an accepted warning."
  (with-temp-buffer
    (insert-file-contents file t)
    (emacs-lisp-mode)
    (dolist (problem (package-lint-buffer))
      (pcase-let ((`(,line ,column ,type ,message) problem))
        (let ((report (format "package-lint: %s:%d:%d: %s: %s"
                              (file-name-nondirectory file) line column type message)))
          (if (and (eq type 'warning)
                   (seq-some (lambda (accepted) (string-prefix-p accepted message)) compmd-lint--accepted-warnings))
              (princ (format "%s (accepted)\n" report))
            (compmd-lint--found report)))))))

(defun compmd-lint--checkdoc (file)
  "Run checkdoc on FILE, counting what it finds."
  (advice-add 'checkdoc-error :before
              (lambda (_point message) (compmd-lint--found (format "checkdoc: %s" message)))
              '((name . compmd-lint)))
  (checkdoc-file file)
  (advice-remove 'checkdoc-error 'compmd-lint))

(defun compmd-lint--byte-compile (file)
  "Byte-compile FILE into a temporary folder, counting a warning or an error."
  (let* ((folder (make-temp-file "compmd-lint-" t))
         (byte-compile-dest-file-function
          (lambda (source) (expand-file-name (concat (file-name-nondirectory source) "c") folder)))
         (byte-compile-error-on-warn t))
    (unless (byte-compile-file file)
      (compmd-lint--found "byte-compile: a warning or an error, above"))
    (delete-directory folder t)))

(let ((file (expand-file-name "../compmd-ts-mode.el" (file-name-directory load-file-name))))
  (compmd-lint--install-package-lint (pop command-line-args-left))
  (compmd-lint--package-lint file)
  (compmd-lint--checkdoc file)
  (compmd-lint--byte-compile file)
  (if (> compmd-lint--problems 0)
      (progn (princ (format "%d found\n" compmd-lint--problems)) (kill-emacs 1))
    (princ "package-lint, checkdoc and byte-compile find nothing\n")
    (kill-emacs 0)))

;;; lint.el ends here
