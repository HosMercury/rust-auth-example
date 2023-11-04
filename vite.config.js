import path from 'path';

export default {
  base: '/',
  root: path.join(__dirname, '/templates/index.html'),
  build: {
    outDir: path.join(__dirname, '/public'),
    rollupOptions: {
      input: path.join(__dirname, '/src/assets/js/main.js'),
    },
    emptyOutDir: true,
  },
};
