
activate.bat py35 && cargo build --release --manifest-path lib-cp35\Cargo.toml &&^
activate.bat py36 && cargo build --release --manifest-path lib-cp36\Cargo.toml &&^
deactivate.bat && ^
copy /Y lib-cp35\target\release\libnastran_cp35.dll nastranrs.cp35-win_amd64.pyd &&^
copy /Y lib-cp36\target\release\libnastran_cp36.dll nastranrs.cp36-win_amd64.pyd

