
const Messages = {
    name: 'messages',
    components: {
    },
    computed: {
        messages() {
            return this.$store.state.messages;
        }
    },
    data() {
        return {
        };
    },
    methods: {
        removeMessage(messageId) {
            this.$store.dispatch('removeMessage', { messageId: messageId })
        }
    },
    template: `
<div class="container">
    <div v-for="m in messages" v-bind:class="'row alert alert-' + m.type" v-bind:key="m.id"
         role="alert">
        <div class="col-11">
            {{ m.message }}
        </div>
        <div class="col-1">
            <button type="button" class="close" aria-label="Close"
                    @click="removeMessage(m.id)">
                <span aria-hidden="true">&times;</span>
            </button>
        </div>
    </div>
</div>
`
}
