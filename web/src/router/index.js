import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '../views/Dashboard.vue'
import Entry from '../views/Entry.vue'
import Exit from '../views/Exit.vue'
import Inventory from '../views/Inventory.vue'
import Records from '../views/Records.vue'
import Statistics from '../views/Statistics.vue'

const routes = [
  { path: '/', name: 'Dashboard', component: Dashboard },
  { path: '/entry', name: 'Entry', component: Entry },
  { path: '/exit', name: 'Exit', component: Exit },
  { path: '/inventory', name: 'Inventory', component: Inventory },
  { path: '/records', name: 'Records', component: Records },
  { path: '/statistics', name: 'Statistics', component: Statistics },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
