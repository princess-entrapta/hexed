import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import Login from './Login.vue'
import Games from './Games.vue'
import Deploy from './Deploy.vue'
import Routerview from './Routerview.vue'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/play/:gameId',
      name: 'home',
      component: App,
      props: true,
    },
    {
      path: '/play',
      name: "list_games",
      component: Games
    },
    {
      path: '/create/:gameId',
      name: "create_game",
      component: Deploy,
      props: true
    },
    {
      path: '/',
      name: 'login',
      component: Login
    }
  ]
})
const app = createApp(Routerview)

app.use(createPinia())
app.use(router)

app.mount('#app')
