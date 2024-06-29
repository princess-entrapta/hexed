import { defineStore } from 'pinia'
import { reactive } from 'vue'
import { useRouter } from 'vue-router'

export class PlayerDeployment {
  entities: { [x: number]: { [y: number]: string | null } } = reactive({})
  drop_tiles: { [x: number]: number } = reactive({})
  allowed_classes: number[] = []
}

export const useDeployStore = defineStore('deploy', {
  state: () => {

    let deployments: PlayerDeployment[] = reactive([])
    let gameId: number = null
    let idx: number | null = null
    let selectedCreature = null
    //var ws = new WebSocket('wss://' + location.host + "/api/" + location.pathname);

    return { idx, gameId, deployments, selectedCreature }
  },
  getters: {
    entityList() {
      let list = [];
      for (let x in this.deployments[this.idx].entities) {
        for (let y in this.deployments[this.idx].entities[x]) {
          if (this.deployments[this.idx].entities[x][y] !== null) {
            list.push({ x: parseInt(x), y: parseInt(y), game_class: this.deployments[this.idx].entities[x][y].game_class })
          }
        }
      }
      console.log(list)
      return list
    },
  },
  actions: {
    connect(game_id: number) {
      this.gameId = game_id
      fetch("/game/" + game_id + '/scenario_players').then((r) => r.json().then((v) => {
        this.deployments = v

        for (let player in this.deployments) {
          this.deployments[player].entities = {}
          this.deployments[player].drop_tiles.map((o) => {
            let o1 = {};
            o1[o.y] = null;
            this.deployments[player].entities[o.x] = o1;
          })
        }
      }))
    },
    deploy() {
      let router = useRouter()
      fetch('/game/' + this.gameId + '/deploy', { method: 'POST', headers: { 'Content-type': 'application/json' }, body: JSON.stringify({ scenario_player_id: this.deployments[this.idx].id, entities: this.entityList }) }).then(() => router.push("/play/" + this.gameId))
    }
  }
}
)

