<template>
  <div>
    <div v-for="(element, index) in settings" :key="element.name">
      <h2 v-if="element.type == null" class="mb-4" :class="{ 'mt-5': index !== 0 }">
        {{ element.name }}
      </h2>
      <div v-else class="mb-4">
        <div class="d-flex mb-2 align-items-center">
          <p class="mb-0 mr-1">{{ element.name }}</p>
          <a v-if="element.tooltip != null" v-b-tooltip.hover :title="element.tooltip">
            <BaseIcon path="help" />
          </a>
        </div>
        <BaseInput
          v-if="element.type === 'input'"
          v-model="newConfig[element.key]"
          :required="element.required === 'true'"
          :placeholder="element.placeholder"
        />
        <BaseInput
          v-else-if="element.type === 'input-number'"
          v-model.number="newConfig[element.key]"
          type="number"
          :required="element.required === 'true'"
          :placeholder="element.placeholder"
        />
        <BaseSelect
          v-else-if="element.type.startsWith('select')"
          v-model="newConfig[element.key]"
          label="name"
          model-key="id"
          :placeholder="getPlaceholder(element)"
          :allow-empty="element.required !== 'true'"
          :multiple="element.type === 'select-multiple'"
          :options="getOptions(element)"
        />
      </div>
    </div>
    <UpdateBar :changed="hasChanges" @save="saveConfig" @reset="newConfig = $api.clone(config)" />
    <h2 class="mt-5 mb-4">
      Danger Zone
      <a v-b-tooltip.hover title="These actions may only be performed by the owner of the server.">
        <p class="mb-0 d-inline-block align-middle"><BaseIcon path="help" /></p>
      </a>
    </h2>
    <BaseButton type="danger" @click="deleteAllData">Delete All Data</BaseButton>
    <BaseButton type="danger" @click="resetStatistics">Reset Statistics</BaseButton>
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  async asyncData({ $getContent }) {
    return {
      settings: await $getContent('settings'),
    }
  },
  data() {
    return {
      newConfig: this.$api.clone(this.$store.getters['guild/config']),
    }
  },
  computed: {
    hasChanges() {
      return !this.$api.isEqual(this.config, this.newConfig)
    },
    ...mapGetters('guild', ['guild', 'config']),
  },
  methods: {
    getOptions(element) {
      if (element.options === 'enable') {
        return [
          { name: 'Enabled', id: true },
          { name: 'Disabled', id: false },
        ]
      }
      if (element.options === 'channels') {
        return this.guild.channels
          .filter(element => element.kind === 0)
          .map(element => ({
            ...element,
            name: `#${element.name}`,
          }))
      }
      if (element.options === 'roles') {
        return this.guild.roles.map(element => element).sort((a, b) => b.position - a.position)
      }
      return []
    },
    getPlaceholder(element) {
      if (element.options === 'channels') {
        return 'Select channel...'
      }
      if (element.options === 'roles') {
        return 'Select roles...'
      }
      return ''
    },
    async saveConfig() {
      const diff = this.$api.diff(this.config, this.newConfig)
      Object.keys(diff).forEach(element => {
        if (element.endsWith('_log') && diff[element] == null) {
          diff[element] = 0
        }
      })
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/settings`, diff)
        .then(() => {
          this.$toast.success('Updated server settings.')
          this.$store.commit('guild/setConfig', this.newConfig)
        })
        .catch(this.$error)
    },
    async deleteAllData() {
      await this.$modal(
        'Do you really want to delete all data? This action cannot be undone. This will permanently delete all information about this server, including playlists, settings, statistics and logs.',
        {
          title: 'Confirmation',
          okTitle: 'Delete',
          okVariant: 'danger',
        }
      ).then(async res => {
        if (!res) return
        await this.$axios
          .$delete(`/guilds/${this.$route.params.id}`)
          .then(() => this.$toast.success('Deleted all data about the server.'))
          .catch(this.$error)
      })
    },
    async resetStatistics() {
      await this.$modal(
        'Do you really want to reset the statistics? This action cannot be undone. This will permanently delete all statistics collected about this server.',
        {
          title: 'Confirmation',
          okTitle: 'Delete',
          okVariant: 'danger',
        }
      ).then(async res => {
        if (!res) return
        await this.$axios
          .$delete(`/guilds/${this.$route.params.id}/stats`)
          .then(() => this.$toast.success('Reset the server statistics.'))
          .catch(this.$error)
      })
    },
  },
}
</script>
