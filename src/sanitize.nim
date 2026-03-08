import std/[strutils, xmltree, streams, htmlparser]

const
  AllowedTags = [
    "p", "br", "hr", "div", "span",
    "h1", "h2", "h3", "h4", "h5", "h6",
    "ruby", "rt", "rp", "rb",
    "em", "strong", "b", "i", "u", "s", "sub", "sup",
  ]
  CleanContentTags = ["script", "style", "title", "noscript", "template"]

proc isAllowedTag(tag: string): bool =
  for t in AllowedTags:
    if t == tag: return true
  false

proc isCleanContentTag(tag: string): bool =
  for t in CleanContentTags:
    if t == tag: return true
  false

proc sanitizeNode(node: XmlNode, output: var string) =
  case node.kind
  of xnText:
    output.add xmltree.escape(node.text)
  of xnElement:
    let tag = node.tag.toLowerAscii()
    if isCleanContentTag(tag):
      return  # Remove tag and all content
    if isAllowedTag(tag):
      if tag in ["br", "hr"]:
        output.add "<" & tag & ">"
      else:
        output.add "<" & tag & ">"
        for child in node:
          sanitizeNode(child, output)
        output.add "</" & tag & ">"
    else:
      # Not allowed: keep text content only
      for child in node:
        sanitizeNode(child, output)
  of xnComment:
    discard  # Strip comments
  else:
    discard

proc processChildren(node: XmlNode, output: var string) =
  for child in node:
    if child.kind == xnElement:
      if child.tag.toLowerAscii() in ["html", "head", "body"]:
        processChildren(child, output)
      else:
        sanitizeNode(child, output)
    elif child.kind == xnText:
      output.add xmltree.escape(child.text)

proc clean*(html: string): string =
  result = ""
  if html.len == 0:
    return
  let wrapped = "<body>" & html & "</body>"
  try:
    let doc = parseHtml(newStringStream(wrapped))
    processChildren(doc, result)
  except:
    result = xmltree.escape(html)
