# RTP

<https://www.rfc-editor.org/rfc/rfc3550#page-12>

Stuff is sent in network byte order, i.e. big-endian.

To be able to decrypt RTSP frames in Wireshark, we can use `ffplay`:

```bash
export SSLKEYLOGFILE=~/ssl-keys.log
ffplay "rtsps://bblp:192190e7@192.168.0.96:322/streaming/live/1"
```

Then go into the Wireshark TLS settings and select the log file to use.

## Payload type 96: "dynamic"

<https://www.iana.org/assignments/rtp-parameters/rtp-parameters.xhtml>,
[RFC3551](https://www.rfc-editor.org/rfc/rfc3551.html).

## Header

The original diagram in the spec is really annoying as it works in base 10. I want bytes! So here
they are:

```
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|V=2|P|X|  CC   |M|     PT      |       sequence number         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           timestamp                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           synchronization source (SSRC) identifier            |
+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
|            contributing source (CSRC) identifiers             |
|                             ....                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

It seems that most frames apart from a couple at the start begin with `80 60 c3`, then what I
_reckon_ is the low byte of the sequence counter in the next byte. This smells like an RTP header,
although none of the fixed values (like `V=2`) seem to match up. The preceding stuff like
`24 00 05 ac` might be some kind of frame delimiter? Or some custom stuff by the LIVE555 streaming
crap? I dunno.

# Figuring out the stream format

The normal `00 00 00 01` NALU packet delimiter doesn't exist in any of the data I've captured from
either my Rust code or `ffplay`.

A Wireshark dump shows small frames like this from `ffplay`:

```
24 00 05 ac
```

`05 ac` is `1452` in decimal, which is the length of the next chunk of data captured. Does this mean
that the stream is in fact AVCC? The `24 00` part is still a mystery.

Lining up `24 00` with the header diagram above we get:

```
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
|V=2|P|X|  CC   |M|     PT      |

Rust print:
 0 0 1 0 0 1 0 0 0 0 0 0 0 0 0 0

Each byte mirrored:
 0 0 1 0 0 1 0 0 0 0 0 0 0 0 0 0
```

Which doesn't make much sense...

What about `80 60`?

```
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
|V=2|P|X|  CC   |M|     PT      |

Rust print:
 1 0 0 0 0 0 0 0 0 1 1 0 0 0 0 0

Each byte mirrored:
 0 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0
```

---

Every chunk seems to have a common header of `80, 60, 5d, a2, 4a, d4, 97, f9, d4, 9e, 7f, e1` after
the weird 4 byte header.

---

According to <https://stackoverflow.com/a/75748093>:

> AVCC is best option for: a stored file (with known sizes & offsets).

But how much do you want to bet that Bambu just threw it into their live streaming setup? The
LIVE555 website implies it prefers streaming files, so maybe that's why.

First 4 bytes of an AVCC frame are called "extradata" or "sequence header"
<https://stackoverflow.com/a/24890903>

---

From ffmpeg/ffplay in the DESCRIBE response:

```
v=0
o=- 1723111495901673 1 IN IP4 192.168.0.96
s=rtsp stream server
i=Thu Aug  8 11:04:55 2024

t=0 0
a=tool:LIVE555 Streaming Media v2023.03.30
a=type:broadcast
a=control:*
a=range:npt=now-
a=x-qt-text-nam:rtsp stream server
a=x-qt-text-inf:Thu Aug  8 11:04:55 2024

m=video 0 RTP/AVP 96
c=IN IP4 0.0.0.0
b=AS:17186
a=rtpmap:96 H264/90000
a=fmtp:96 packetization-mode=1;profile-level-id=42C01F;sprop-parameter-sets=Z0LAH42NUCSC2TZAAAADAEAAAA8jwiEagA==,aM4xsg==
a=control:track1
```

This is an SDP (`application/sdp`) formatted response. I can test them with VLC:
<https://stackoverflow.com/q/20634476>

The last little base64 `aM4xsg==` decodes to (hex):

```
68,CE,31,B2
```

This is present in the bytes I get in the Rust code! No idea what this means...

---

The longer `Z0LAH42NUCSC2TZAAAADAEAAAA8jwiEagA==` (SPS and PPS?) is:

```
67,42,C0,1F,8D,8D,50,24,82,D9,36,40,00,00,03,00,40,00,00,0F,23,C2,21,1A,80
```

which is indeed present as well.

Some more info here <https://www.cardinalpeak.com/blog/the-h-264-sequence-parameter-set>

The received frame starts like this:

```
[ 24, 00, 00, 25 ], 80, 60, 07, 8f, 84, b4, 4d, 39, 5b, 6e, 5f, 94, [ 67, 42, c0, 1f, 8d, 8d, 50, 24, 82, d9, 36, 40, 00, 00, 03, 00, 40, 00, 00, 0f, 23, c2, 21 ]
```

---

`ffmpeg` says

```
Input #0, rtsp, from 'rtsps://bblp:192190e7@192.168.0.96:322/streaming/live/1':
  Metadata:
    title           : rtsp stream server
    comment         : Sat Aug 10 00:25:18 2024
  Duration: N/A, start: 0.031978, bitrate: N/A
  Stream #0:0, 21, 1/90000: Video: h264 (Constrained Baseline), 1 reference frame, yuvj420p(pc, progressive, left), 1168x720, 0/1, 30 fps, 30 tbr, 90k tb
```

Profile is `66d` with constrained bit set to 1
