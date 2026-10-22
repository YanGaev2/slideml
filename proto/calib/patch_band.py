p = 'F:/pptx-ai-native/proto/src/deck.rs'
s = open(p, encoding='utf-8').read()
old = '''                let (g, overlays) = crate::chart::chart(c, ctx, pkg, &th.body)?;
                (format!(
                    "<p:graphicFrame>'''
new = r'''                let (g, overlays) = crate::chart::chart(c, ctx, pkg, &th.body)?;
                let (under, overlays) = overlays.split_once('{1}').map(|(u, o)| (u.to_string(), o.to_string())).unwrap_or((String::new(), overlays));
                (format!(
                    "{under}<p:graphicFrame>'''
assert old in s
s = s.replace(old, new, 1)
open(p, 'w', encoding='utf-8').write(s)
print('ok')
