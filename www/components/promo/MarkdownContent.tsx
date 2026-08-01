"use client";

import type { ReactNode } from "react";

interface MarkdownContentProps {
  text: string;
}

/** Lightweight markdown renderer — supports code blocks, inline code, bold, links, paragraphs */
export function MarkdownContent({ text }: MarkdownContentProps) {
  const nodes = parseMarkdown(text);
  return <>{nodes}</>;
}

/* ─── Parser ─────────────────────────────────────────── */

interface MdNode {
  type: "text" | "bold" | "inline_code" | "code_block" | "link" | "br";
  content?: string;
  lang?: string;
  children?: MdNode[];
}

function parseMarkdown(text: string): ReactNode[] {
  const elements: ReactNode[] = [];
  let i = 0;

  while (i < text.length) {
    // Fenced code block
    if (text.startsWith("```", i)) {
      const end = text.indexOf("```", i + 3);
      if (end !== -1) {
        const blockStart = i + 3;
        let lang = "";
        let codeStart = blockStart;

        // Check for language tag
        const newlineIdx = text.indexOf("\n", blockStart);
        if (newlineIdx !== -1 && newlineIdx < end) {
          const possibleLang = text.slice(blockStart, newlineIdx).trim();
          if (possibleLang && !possibleLang.includes(" ")) {
            lang = possibleLang;
            codeStart = newlineIdx + 1;
          }
        }

        const code = text.slice(codeStart, end);
        elements.push(
          <CodeBlock key={`cb-${i}`} lang={lang} code={code} />
        );
        i = end + 3;
        // Skip trailing newline after code block
        if (text[i] === "\n") i++;
        continue;
      }
    }

    // Double newline → paragraph break
    if (text[i] === "\n" && text[i + 1] === "\n") {
      elements.push(<div key={`br-${i}`} className="h-3" />);
      i += 2;
      continue;
    }

    // Single newline
    if (text[i] === "\n") {
      elements.push(<br key={`nl-${i}`} />);
      i++;
      continue;
    }

    // Find end of current line
    const nextNL = text.indexOf("\n", i);
    const lineEnd = nextNL === -1 ? text.length : nextNL;

    // Parse inline elements within this line
    const lineText = text.slice(i, lineEnd);
    elements.push(
      <span key={`line-${i}`}>{parseInline(lineText)}</span>
    );

    i = lineEnd;
    // Don't consume the newline — let next iteration handle it
  }

  return elements;
}

function parseInline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  let i = 0;

  while (i < text.length) {
    // Bold **...**
    if (text[i] === "*" && text[i + 1] === "*") {
      const end = text.indexOf("**", i + 2);
      if (end !== -1) {
        nodes.push(
          <strong key={`b-${i}`} className="font-bold text-white">
            {text.slice(i + 2, end)}
          </strong>
        );
        i = end + 2;
        continue;
      }
    }

    // Link [...](...)
    if (text[i] === "[") {
      const closeBracket = text.indexOf("](", i);
      const closeParen = text.indexOf(")", closeBracket);
      if (closeBracket !== -1 && closeParen !== -1) {
        const linkText = text.slice(i + 1, closeBracket);
        const url = text.slice(closeBracket + 2, closeParen);
        nodes.push(
          <a
            key={`link-${i}`}
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-emerald-400 underline decoration-emerald-400/30 hover:decoration-emerald-400"
          >
            {linkText}
          </a>
        );
        i = closeParen + 1;
        continue;
      }
    }

    // Inline code `...`
    if (text[i] === "`") {
      const end = text.indexOf("`", i + 1);
      if (end !== -1) {
        nodes.push(
          <code
            key={`ic-${i}`}
            className="px-1.5 py-0.5 rounded bg-[#3b4261]/60 text-[#e0af68] text-[0.9em] font-mono"
          >
            {text.slice(i + 1, end)}
          </code>
        );
        i = end + 1;
        continue;
      }
    }

    // Regular text — consume until next special char or end
    let j = i;
    while (j < text.length) {
      if (text[j] === "*" || text[j] === "`" || text[j] === "[") break;
      j++;
    }
    if (j > i) {
      nodes.push(<span key={`t-${i}`}>{text.slice(i, j)}</span>);
      i = j;
    } else {
      // Single special character that didn't match, emit as text
      nodes.push(<span key={`tc-${i}`}>{text[i]}</span>);
      i++;
    }
  }

  return nodes;
}

/* ─── Code Block with syntax highlighting ────────────── */

const PYTHON_KEYWORDS = new Set([
  "import", "from", "def", "class", "return", "if", "elif", "else",
  "while", "for", "in", "not", "and", "or", "True", "False", "None",
  "try", "except", "finally", "with", "as", "pass", "break", "continue",
  "self", "__init__", "__name__", "__main__",
]);

const BASH_KEYWORDS = new Set([
  "pip", "install", "python", "oneinit",
]);

function highlightLine(line: string, lang: string): ReactNode {
  const keywords = lang === "bash" ? BASH_KEYWORDS : PYTHON_KEYWORDS;

  // Simple tokenizer for Python
  const tokens: ReactNode[] = [];
  let i = 0;

  while (i < line.length) {
    // String literals
    if (line[i] === '"' || line[i] === "'") {
      const quote = line[i];
      let j = i + 1;
      while (j < line.length && line[j] !== quote) {
        if (line[j] === "\\") j++;
        j++;
      }
      tokens.push(
        <span key={`s-${i}`} className="text-[#9ece6a]">
          {line.slice(i, j + 1)}
        </span>
      );
      i = j + 1;
      continue;
    }

    // Comments
    if (line[i] === "#") {
      tokens.push(
        <span key={`c-${i}`} className="text-[#565f89] italic">
          {line.slice(i)}
        </span>
      );
      break;
    }

    // Numbers
    if (/[0-9]/.test(line[i])) {
      let j = i;
      while (j < line.length && /[0-9.]/.test(line[j])) j++;
      tokens.push(
        <span key={`n-${i}`} className="text-[#ff9e64]">{line.slice(i, j)}</span>
      );
      i = j;
      continue;
    }

    // Words (identifiers + keywords)
    if (/[a-zA-Z_]/.test(line[i])) {
      let j = i;
      while (j < line.length && /[a-zA-Z0-9_]/.test(line[j])) j++;
      const word = line.slice(i, j);
      if (keywords.has(word)) {
        tokens.push(
          <span key={`kw-${i}`} className="text-[#bb9af7]">{word}</span>
        );
      } else {
        tokens.push(<span key={`id-${i}`}>{word}</span>);
      }
      i = j;
      continue;
    }

    // Everything else (operators, punctuation)
    tokens.push(<span key={`op-${i}`} className="text-[#89ddff]">{line[i]}</span>);
    i++;
  }

  return <>{tokens}</>;
}

function CodeBlock({ lang, code }: { lang: string; code: string }) {
  const lines = code.split("\n");

  return (
    <div className="my-2 rounded-lg overflow-hidden border border-[#3b4261]/50 bg-[#0f0f1a]">
      {/* Header bar */}
      {lang && (
        <div className="flex items-center justify-between px-4 py-1.5 bg-[#24283b]/60 border-b border-[#3b4261]/30">
          <span className="text-[10px] uppercase tracking-widest text-[#565f89] font-mono">
            {lang}
          </span>
        </div>
      )}
      {/* Code lines */}
      <div className="overflow-x-auto">
        <pre className="p-4 text-xs md:text-sm leading-relaxed font-mono text-[#a9b1d6]">
          <code>
            {lines.map((line, idx) => (
              <div key={idx} className="table-row">
                <span className="table-cell text-right pr-4 select-none text-[#3b4261] w-8">
                  {idx + 1}
                </span>
                <span className="table-cell">
                  {lang ? highlightLine(line, lang) : line}
                </span>
              </div>
            ))}
          </code>
        </pre>
      </div>
    </div>
  );
}
