(function() {var implementors = {};
implementors["bytes"] = [{"text":"impl FromIterator&lt;u8&gt; for Bytes","synthetic":false,"types":[]},{"text":"impl FromIterator&lt;u8&gt; for BytesMut","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; FromIterator&lt;&amp;'a u8&gt; for BytesMut","synthetic":false,"types":[]}];
implementors["futures_util"] = [{"text":"impl&lt;F:&nbsp;Future&gt; FromIterator&lt;F&gt; for JoinAll&lt;F&gt;","synthetic":false,"types":[]},{"text":"impl&lt;Fut:&nbsp;Future + Unpin&gt; FromIterator&lt;Fut&gt; for SelectAll&lt;Fut&gt;","synthetic":false,"types":[]},{"text":"impl&lt;F:&nbsp;TryFuture&gt; FromIterator&lt;F&gt; for TryJoinAll&lt;F&gt;","synthetic":false,"types":[]},{"text":"impl&lt;Fut:&nbsp;TryFuture + Unpin&gt; FromIterator&lt;Fut&gt; for SelectOk&lt;Fut&gt;","synthetic":false,"types":[]},{"text":"impl&lt;Fut:&nbsp;Future&gt; FromIterator&lt;Fut&gt; for FuturesOrdered&lt;Fut&gt;","synthetic":false,"types":[]},{"text":"impl&lt;Fut&gt; FromIterator&lt;Fut&gt; for FuturesUnordered&lt;Fut&gt;","synthetic":false,"types":[]},{"text":"impl&lt;St:&nbsp;Stream + Unpin&gt; FromIterator&lt;St&gt; for SelectAll&lt;St&gt;","synthetic":false,"types":[]}];
implementors["proc_macro2"] = [{"text":"impl FromIterator&lt;TokenTree&gt; for TokenStream","synthetic":false,"types":[]},{"text":"impl FromIterator&lt;TokenStream&gt; for TokenStream","synthetic":false,"types":[]}];
implementors["syn"] = [{"text":"impl&lt;T, P&gt; FromIterator&lt;T&gt; for Punctuated&lt;T, P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: Default,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;T, P&gt; FromIterator&lt;Pair&lt;T, P&gt;&gt; for Punctuated&lt;T, P&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()