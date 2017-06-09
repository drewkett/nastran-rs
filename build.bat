
activate.bat py35 && ^
cargo build --release --example nastranrs-cp35 --features="cpython/python-3-5" &&^
activate.bat py36 && ^
cargo build --release --example nastranrs-cp36 --features="cpython/python-3-6" &&^
deactivate.bat && ^
copy target\release\examples\nastranrs-cp35.dll nastranrs.cp35-win_amd64.pyd &&^
copy target\release\examples\nastranrs-cp36.dll nastranrs.cp36-win_amd64.pyd

