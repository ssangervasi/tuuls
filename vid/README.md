# Terminal Video Player
```
       xxxxxx  xxx      xxxxxxx     xxxxxxxxxxxxxxxxxxxxxxxx   --   ----                 -    -     xxxx
       xxxxxx- x     x━━━━━━━━━━━xx  xxxxxxxxxxxxxxxxxxxxxxx   --    -----                -   -     xxxx
       xxxxxx-   xxxx━✖✖✖✖✖✖✖✖✖✖✖✖━x  xxxxxxxxxxxxxxxxxxxxxx   --    --  x━━━x             - --     xxxx
       xxxxxx---x━━xx━━✖✖✖✖✖✖✖✖✖✖✖✖━x  xxxxxxxxxx   x━xxx━━━x  --      x━━✖✖✖━━━━━xxxxxx     ----   xxxx
       xxxxxx- · ━━━xxx━✖━━━━✖✖✖✖✖✖━━x xxxxxx -          xxx   --    x✖✖━━✖✖━━━━✖✖✖✖✖━x      ---    xxxx
       xxxxx  x-x━━━━xx━━━━━━━━━━✖✖━━  xxxxx -  ----------     --  --━✖━━━━━━✖✖━━━✖✖✖x       --     xxxx
       xxxxx  x x━━━━x━━━━━━✖━x━✖✖✖━━       - xx  ---  ----    --  - x━━━━━━x✖✖✖✖━━━━        --     xxxx
       xxxxx  x- xxxxxx━━━━━━━━━✖✖✖━━ ---------      x    --   -- --x━━━━━━━━━━━━━━x         --     xxxx
       xxxxx  x·-━━━━━━━━━━━━━━━━━━━━━  -------------  -  --   ---- ━━━━━━━━━━━━━━x  xx━━x    -     xxxx
       xxxxx  x  ━━━━━━━━━━━━━✖━━━━━━━   x          --         -- x━━━━━━━━━━━━━✖✖━━━━━━━x    -     xxxx
       xxxxx      x━━━━━━━━━━━━━━✖✖✖✖✖━x xxx xxxx x   xx x x   -- ━━━✖━━━━━━━━━✖✖✖✖✖✖━━━x     -     xxxx
       xxxxx  x   x━━━━━━━━━━x━━━━━━━x  -   -   x x---xx x     --   x━━━━━━━━x━✖✖✖✖━━━x---   --     xxxx
       xxxxxx x   x━✖✖✖━━━━━━━━✖✖✖✖━--·-- --·---  x---   ---   --  -- x━━━━━━━━✖✖✖✖━                xxxx
       xxxxx     -x━━━━━━✖✖✖✖✖✖✖✖✖━ --                         -    ---━━✖✖✖✖✖✖✖✖✖✖━---             xxxx
       xxxxxx  -· -━✖━━━━━━━━━━xx━━━ --                        -    ---x━✖✖✖✖✖✖✖✖━━x--        -     xxxx
       xxxxx -·    ·x✖✖✖✖✖✖✖✖x-··-x✖━····--                    --      xx━━━━━━xxxxxxxx       -     xxxx
       xxxxx ·      ·-━✖✖✖✖✖━━ -·· ━━ ·  ···--                 -- x  xxxx        xx━✖✖x --    -     xxxx
       xxxxx · ··     · ━✖✖✖✖✖✖x··x✖✖━·  ·····--             --- xx  xxx        x━✖✖x      ---      xxxx
       xxxxx ·····      · ━✖✖✖✖x··-━✖━·  · xxx ·-     xxxxxx -- xx  xx        x━✖✖x      ------     xxxx
       xxxxx ·········   ·-x✖✖✖ ··· ✖━·· ·x✖✖━ ·· xxxxxxxxx ---xx xxxx      x━✖✖━     --       -  - xxxx
       xxxxx ·········  ···· ━━-····xx·   ━✖✖x· ··-        --- ━━xxxxx    xx✖✖✖━     -            - xxxx
       xxxxx · ········· ····- ·····--· ·━✖✖━-  ···- xxxx  -- ━✖✖✖✖✖✖✖━━━━✖✖✖✖x     -            -- xxxx
       xxxxx · ························ x✖✖✖x·  ····-     -- x✖✖✖✖✖✖✖✖✖✖✖✖✖✖✖━      -              x xxx
       xxxxx · ·······················x✖✖✖✖✖━- ······- xx -- ━✖✖✖✖✖✖✖✖✖✖✖✖✖✖✖x     -                 xxx
       xxxxx · ················· ··-·- xx xx━-········ xx --x✖✖✖✖✖✖✖✖✖✖✖✖✖✖✖━      -                xxxx
       xxxxx · ···················x━xx  --------······-xx - ━✖✖✖✖✖✖✖✖✖✖✖✖✖✖✖x      -                xxxx
       xxxxx · ················  · ━✖✖━xx --- -x ···  ·x━ - ━✖✖✖✖✖✖✖✖✖✖✖✖✖✖━x-     -                xxx━
       xxxxx · ················   ·━✖✖━━✖━  ━x-x✖x··  ·x━x- ✖✖✖✖✖✖✖✖✖✖✖✖✖✖✖━ --    -               -xxx8
 

```

## CLI

```
Vid Term 

USAGE:
    vid [FLAGS] <path>

FLAGS:
    -h, --help       Prints help information
    -s               Play sounds using the terminal bell
    -V, --version    Prints version information
    -w               Display a crude audio waveform next to the video

ARGS:
    <path>    Path to video to play
```


## Dependencies

Besides running `cargo install`, this crate depends on `rust-ffmpeg`. This 
requires an existing installation of ffmpeg, as documented [here](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building).

Works like a charm in Linux. I got it to work on Windows but I've already forgotten the environment variables needed to point to the right ffmpeg installation.
