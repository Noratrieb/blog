+++
title = "Having fun with OpenSSH private keys"
date = "2024-09-13"
author = "Noratrieb"
authorTwitter = "@Noratrieb"
tags = ["ssh"]
keywords = ["SSH"]
description = "An interactive way to have fun with OpenSSH private keys"
showFullContent = false
readingTime = true
hideComments = false
draft = false
+++

you likely have an SSH private key.
and unless you're doing something *seriously* wrong, only you have this key.
that's the entire point, after all.

this private key is used for authenticating with an SSH server.
you sign a message with your private key and the server then verifies it with the public key.
this ensures that it's you who authenticated to the server, and not your friends or enemies.
or me. don't let me into your servers.

but how are these keys encoded and can we have fun with them?
let's use `ssh-keygen -t ed25519` to generate a test key and find out.

```
$ ssh-keygen -t ed25519
Generating public/private ed25519 key pair.
Enter file in which to save the key (/home/nora/.ssh/id_ed25519): testkey
Enter passphrase (empty for no passphrase): 
Enter same passphrase again: 
Your identification has been saved in testkey
Your public key has been saved in testkey.pub
The key fingerprint is:
SHA256:IPrdC+4S0ZIzwS1oYN3A78Q29yV6gpDgiEkPwJtj0Wc nora@nixos
The key's randomart image is:
+--[ED25519 256]--+
|=o++o.           |
|.*o++E.          |
|=oB B=.          |
|+* =*B.o . .     |
|. o ==+ S o      |
|   ..+ + o       |
|    ..o +        |
|    .. . .       |
|     oo .        |
+----[SHA256]-----+
```

this command has created two files, `testkey` and `testkey.pub`.

`testkey.pub` contains the public key and looks like this. if you're following along at home it probably looks different.
hopefully. unless you have gotten very lucky, which would be terrible and the downfall of the entire cryptographic ecosystem.

```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEc5o2i/B1bVs7X2dJjE48l7fqAyMdgrbAItrO8XWwP9 nora@nixos
```

the public key starts with the key type, ed25519.
Ed25519 is a signature algorithm that is commonly used for modern SSH keys.
it's also the default for modern OpenSSH versions, so passing that `-t` flag was unnecessary.
other common algorithms are ECDSA (`ecdsa-sha2-nistp256`) and RSA (`ssh-rsa`).
unless you need compatiblity with ancient servers or are bound by outdated regulation, you probably don't need either of those.

after the key type, we have a base64 encoded blob of the "wire encoding" of the key.
this encoding is [standardized](https://datatracker.ietf.org/doc/html/rfc8709) and is sent by the client to the server every time it wants to authenticate, to choose which key to use.
the exact details vary by key type but for Ed25519, it contains the following:

|bytes|meaning|
------|-------|
|`0000 000b` | name length, 11 |
|`7373 682d 6564 3235 3531 39` | ssh-ed25519 |
|`0000 0020` | encoded Ed25519 public key length, always 32 |
|`4739 a368 bf07 56d5 b3b5 f674 98c4 e3c9 7b7e a032 31d8 2b6c 022d acef 175b 03fd` | encoded Ed25519 public key |

the last part is the comment. it's automatically set to my username and my hostname (i use nixos btw) and can be set to anything with the `-C` parameter.
it's supposed to help us figure out what the key is.

the public key is fairly boring, so we're gonna take a look at the exciting private key instead.

```
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBHOaNovwdW1bO19nSYxOPJe36gMjHYK2wCLazvF1sD/QAAAJCFCe+ShQnv
kgAAAAtzc2gtZWQyNTUxOQAAACBHOaNovwdW1bO19nSYxOPJe36gMjHYK2wCLazvF1sD/Q
AAAEDmrbLtUasQVBfkJV0ILoxDox64ngUwOASQbc8N0oZzNEc5o2i/B1bVs7X2dJjE48l7
fqAyMdgrbAItrO8XWwP9AAAACm5vcmFAbml4b3MBAgM=
-----END OPENSSH PRIVATE KEY-----
```

it goes without saying but never share your private key on the internet and this is obviously just a test key!

the entire key is base64-encoded in the [PEM](https://en.wikipedia.org/wiki/Privacy-Enhanced_Mail) format.
this makes it easier to copy around compared to raw bytes. not that you're supposed to copy it to random places.

an OpenSSH private key consists of two areas:
- a plaintext area with the public key
- a potentially-encrypted area with the private key

most strings are length-prefixed, i'm not gonna mention the length explicitly for many of the cases here.
if it starts with 3 null bytes, the first 4 bytes are probably the length.
for an `ssh-ed25519` key, the format looks like this:

|bytes|meaning|
------|-------|
`6f70 656e 7373 682d 6b65 792d 7631 00` | openssh-key-v1 (null-terminated) |
`0000 0004 6e6f 6e65` | cipher, `none` in this case (`aes256-ctr` is common for encrypted keys) |
`0000 0004 6e6f 6e65` | key derivation function, `none` in this case (`bcrypt` is common for encrypted keys) |
`0000 0000` | key derivation options, empty here (contains the salt and cost for `bcrypt`) |
`0000 0001` | amount of keys, 1 (yes, it could contain multiple) |
a bunch of bytes | the full public key, as seen previously
`0000 0090` | the length of the encrypted part. the rest is encrypted with the previously mentioned cipher and a password |
`8509 ef92 8509 ef92` | two identical 4-byte sequences, to check if decryption was successful |
`0000 000b 7373 682d 6564 3235 3531 39` | ssh-ed25519, the algorithm of the first key (which might seem familiar) |
`0000 0020 4739 a368 bf07 56d5 b3b5 f674 98c4 e3c9 7b7e a0323 1d8 2b6c 022d acef 175b 03fd` | the raw encoded public key bytes |
`0000 0040` | the length of the next part, which contains the... |
`e6ad b2ed 51ab 1054 17e4 255d 082e 8c43 a31e b89e 0530 3804 906d cf0d d286 7334` | ...raw private key bytes... |
`4739 a368 bf07 56d5 b3b5 f674 98c4 e3c9 7b7e a0323 1d8 2b6c 022d acef 175b 03fd` | ...and the public key bytes. AGAIN. YES.

the unencrypted public area makes it easy to check which public key a private key belongs to without needing to enter a password to decrypt it.
the encrypted area makes sure that even if someone manages to steal your private key, they can't use it unless they know your password.
unless you haven't set a password of course. which is why you should set a password for your private key.

having the public key bytes in there THREE TIMES seems very silly. but the fact that the public key is in there at all is useful.

maybe you've been in a situation where you've needed to find the public key file of a private key you had around, and just couldn't find it.
but as I just mentioned, you don't actually need the `.pub` file for that, as the public key is contained in the private key.
`ssh-keygen` can even extract it for you with `-y`!

```
$ ssh-keygen -y -f testkey
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEc5o2i/B1bVs7X2dJjE48l7fqAyMdgrbAItrO8XWwP9 nora@nixos
```

i have a public key.
You can find it on <https://github.com/Noratrieb.keys> (this works for any GitHub user that has uploaded SSH keys!) and at the time of writing, it was
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG0n1ikUG9rYqobh7WpAyXrqZqxQoQ2zNJrFPj12gTpP
```

but you don't care about this, do you? you really want my private key. i know it.
well, here it is:
```
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtz
c2gtZWQyNTUxOQAAACBtJ9YpFBva2KqG4e1qQMl66masUKENszSaxT49doE6TwAA
AIgQ5LRcEOS0XAAAAAtzc2gtZWQyNTUxOQAAACBtJ9YpFBva2KqG4e1qQMl66mas
UKENszSaxT49doE6TwAAAEAoBWfFwPJSZQxTNETJRn40Y2XFP2GbW1aAGX+SzP/o
rG0n1ikUG9rYqobh7WpAyXrqZqxQoQ2zNJrFPj12gTpPAAAAAAECAwQF
-----END OPENSSH PRIVATE KEY-----
```

don't believe me? check it yourself!

```
$ ssh-keygen -y -f the-just-posted-public-key
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG0n1ikUG9rYqobh7WpAyXrqZqxQoQ2zNJrFPj12gTpP
```

it's true! you indeed have my private key! don't do bad things with it, please.

well, you probably won't believe me. you know how SSH private keys and `ssh-keygen -y` works,
and you know that the private key i posted above is just a random private key with my public key put into the public key part.
and you're right. good job!

but maybe your friends don't know that. or your enemies.
posting "your public key" may confuse them and is fun... and we're here for fun!

you can use the generator below to generate a fake private key for a public key.
it only supports `ssh-ed25519` and `ecdsa-sha2-nistp256`.
[no `ssh-rsa`, sorry](https://blog.trailofbits.com/2019/07/08/fuck-rsa/).
if you have an RSA key, get a better key first.
the implementation is based on [cluelessh](https://github.com/Noratrieb/cluelessh), my own SSH toolkit, compiled to WebAssembly.

## generator

<label for="public-key-input">public Key</label>
<br>

<textarea id="public-key-input" rows="10" cols="29"></textarea>
<button id="convert-button" style="margin-left: 10px;">Generate</button>
<div id="public-key-error"></div>

<label for="fake-key-output">fake private key</label>
<textarea id="fake-key-output" rows="10" disabled></textarea>
<style>
#fake-key-output { width: 90vw; }
@media (min-width: 600px) {
    #fake-key-output { width: 30em; }
}
@media (min-width: 1000px) {
    #fake-key-output { width: 50em; }
}
</style>

<script type="module">
import init, { generate_fake } from "./fake_openssh_key.js"

init({
    module_or_path: new URL('fake_openssh_key_bg.wasm', import.meta.url)
});

const input = document.getElementById("public-key-input");
const output = document.getElementById("fake-key-output");
const button = document.getElementById("convert-button");
const error = document.getElementById("public-key-error");

button.addEventListener("click", () => {
    const key = input.value;
    error.innerText = "loading";
    try {
        const result = generate_fake(key);
        output.value = result;
        error.innerText = "";
    } catch(e) {
        console.log(key);
        error.innerText = `error: ${e}`;
    }
});
</script>

what are these SSH keys actually used for? SSH of course. but how? oh do i have a blog post for you:
