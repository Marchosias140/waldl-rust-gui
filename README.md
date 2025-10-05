Updates:

- Fixed a typo on the main.rs and Cargo.toml regarding the name of the app.



Instructions:


This work was created as a place to experiment with waldl, and ultimately lead to the here published Rust GUI for wldl.
I chose Rust over Python as I often do because I love cargo for compiling stuff.
The app shows thumbnails and downloads the wallpaper in 4k to your default Downloads folder by clicking on it.
I also changed sxiv for nsxiv, because both the original wldl script and sxiv were from a few years ago.


Compile with:

``` cargo check ```



So you know it compiles.


Then


```cargo run```


if you want to try the binary, or



```cargo build --release```



if you want the fully optimized binary.





The binary will be in the same folder you had the provided files, inside target/release.








Enjoy!


# waldl

Browser [wallhaven](https://wallhaven.cc/) using `sxiv`

### [script showcasing video](https://youtu.be/C7n-34bEdF8)


## Usage
```
waldl <query>
```
> Leave query empty to use `dmenu`

- Select wallpapers by marking them using `m` in `sxiv`.
- Quit `sxiv` using `q`.

Selected images would be downloaded. The default download directory is

	~/.local/share/wallhaven

Defaults can be changed by changing the user variables, in the start of the
script.

## Installation
Default installation path is `/usr/local/bin`, to change it edit the `INSTALL_PATH` variable in the Makefile.

To install `waldl` just run:
```
make install
```


To later uninstall `waldl` run:
```
make uninstall
```

## Dependencies

* sxiv
* jq
* curl
* dmenu ( *optional* )


