cd web-ui
npm run build
cd ..
rm -rf static/*
mkdir -p static/
cp -r web-ui/build/* static/