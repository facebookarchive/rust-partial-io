(function() {var implementors = {};
implementors["bytes"] = [{"text":"impl&lt;B:&nbsp;BufMut + Sized&gt; Write for Writer&lt;B&gt;","synthetic":false,"types":[]}];
implementors["env_logger"] = [{"text":"impl Write for Formatter","synthetic":false,"types":[]}];
implementors["futures_util"] = [{"text":"impl&lt;T&gt; Write for AllowStdIo&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Write,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["partial_io"] = [{"text":"impl&lt;R&gt; Write for PartialRead&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: Read + Write,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;W&gt; Write for PartialWrite&lt;W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Write,&nbsp;</span>","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()