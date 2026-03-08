import unittest
import std/strutils
import ../src/sanitize

suite "sanitize.clean":
  test "preserves allowed tags":
    check clean("<p>hello</p>") == "<p>hello</p>"
    check clean("<em>text</em>") == "<em>text</em>"
    check clean("<strong>bold</strong>") == "<strong>bold</strong>"
    check clean("<ruby>漢<rt>かん</rt></ruby>") == "<ruby>漢<rt>かん</rt></ruby>"

  test "preserves self-closing tags":
    check clean("<br>") == "<br>"
    check clean("<hr>") == "<hr>"

  test "strips script tags and content":
    check clean("<script>alert('xss')</script>") == ""
    check clean("before<script>evil()</script>after") == "beforeafter"

  test "strips style tags and content":
    check clean("<style>body{color:red}</style>") == ""

  test "strips noscript and template tags":
    check clean("<noscript>fallback</noscript>") == ""
    check clean("<template>hidden</template>") == ""

  test "strips disallowed tags but keeps text":
    check clean("<a href='http://example.com'>link</a>") == "link"
    check clean("<div><a>text</a></div>") == "<div>text</div>"

  test "removes img with onerror (XSS vector)":
    check clean("""<img onerror="alert(1)" src="x">""") == ""

  test "strips HTML comments":
    check clean("before<!-- comment -->after") == "beforeafter"

  test "handles empty input":
    check clean("") == ""

  test "escapes plain text":
    check clean("a < b & c > d") == "a &lt; b &amp; c &gt; d"

  test "handles nested allowed tags":
    check clean("<p><em>nested</em></p>") == "<p><em>nested</em></p>"

  test "strips all attributes from allowed tags":
    check clean("""<p class="foo" id="bar">text</p>""") == "<p>text</p>"
    check clean("""<div style="color:red">text</div>""") == "<div>text</div>"

  test "removes event handler attributes (XSS prevention)":
    check clean("""<p onclick="alert(1)">text</p>""") == "<p>text</p>"

  test "handles mixed allowed and disallowed tags":
    check clean("<p><script>bad</script>good</p>") == "<p>good</p>"

  test "removes title tag with content":
    check clean("<title>悪意のあるタイトル</title>") == ""
